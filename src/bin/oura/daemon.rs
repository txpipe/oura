use gasket::daemon::Daemon;
use oura::{cursor, filters, framework::*, sinks, sources};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{sync::Arc, time::Duration};
use tracing::info;

use crate::console;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub address: Option<String>,
}

#[derive(Deserialize)]
struct ConfigRoot {
    source: sources::Config,
    filters: Option<Vec<filters::Config>>,
    sink: sinks::Config,
    intersect: IntersectConfig,
    finalize: Option<FinalizeConfig>,
    chain: Option<ChainConfig>,
    retries: Option<gasket::retries::Policy>,
    cursor: Option<cursor::Config>,
    metrics: Option<MetricsConfig>,
}

impl ConfigRoot {
    pub fn new(explicit_file: &Option<std::path::PathBuf>) -> Result<Self, config::ConfigError> {
        let mut s = config::Config::builder();

        // our base config will always be in /etc/scrolls
        s = s.add_source(config::File::with_name("/etc/oura/daemon.toml").required(false));

        // but we can override it by having a file in the working dir
        s = s.add_source(config::File::with_name("oura.toml").required(false));

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file.as_ref().and_then(|x| x.to_str()) {
            s = s.add_source(config::File::with_name(explicit).required(true));
        }

        // finally, we use env vars to make some last-step overrides
        s = s.add_source(config::Environment::with_prefix("OURA").separator("_"));

        s.build()?.try_deserialize()
    }
}

fn define_gasket_policy(config: Option<&gasket::retries::Policy>) -> gasket::runtime::Policy {
    let default_policy = gasket::retries::Policy {
        max_retries: 20,
        backoff_unit: Duration::from_secs(1),
        backoff_factor: 2,
        max_backoff: Duration::from_secs(60),
        dismissible: false,
    };

    gasket::runtime::Policy {
        tick_timeout: None,
        bootstrap_retry: config.cloned().unwrap_or(default_policy.clone()),
        work_retry: config.cloned().unwrap_or(default_policy.clone()),
        teardown_retry: config.cloned().unwrap_or(default_policy.clone()),
    }
}

fn connect_stages(
    mut source: sources::Bootstrapper,
    mut filters: Vec<filters::Bootstrapper>,
    mut sink: sinks::Bootstrapper,
    mut cursor: cursor::Bootstrapper,
    policy: gasket::runtime::Policy,
) -> Result<Daemon, Error> {
    let mut prev = source.borrow_output();

    for filter in filters.iter_mut() {
        gasket::messaging::tokio::connect_ports(prev, filter.borrow_input(), 100);
        prev = filter.borrow_output();
    }

    gasket::messaging::tokio::connect_ports(prev, sink.borrow_input(), 100);
    let prev = sink.borrow_cursor();

    gasket::messaging::tokio::connect_ports(prev, cursor.borrow_track(), 100);

    let mut tethers = vec![];
    tethers.push(source.spawn(policy.clone()));
    tethers.extend(filters.into_iter().map(|x| x.spawn(policy.clone())));
    tethers.push(sink.spawn(policy.clone()));
    tethers.push(cursor.spawn(policy));

    let runtime = Daemon(tethers);

    Ok(runtime)
}

fn setup_tracing() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();
}

async fn serve_prometheus(
    daemon: Arc<Daemon>,
    metrics: Option<MetricsConfig>,
) -> Result<(), Error> {
    if let Some(metrics) = metrics {
        info!("starting metrics exporter");
        let runtime = daemon.clone();

        let addr: SocketAddr = metrics
            .address
            .as_deref()
            .unwrap_or("0.0.0.0:9186")
            .parse()
            .map_err(Error::parse)?;

        gasket_prometheus::serve(addr, runtime).await;
    }

    Ok(())
}

pub fn run(args: &Args) -> Result<(), Error> {
    if !args.tui {
        setup_tracing();
    }

    let config = ConfigRoot::new(&args.config).map_err(Error::config)?;

    let chain = config.chain.unwrap_or_default();
    let intersect = config.intersect;
    let finalize = config.finalize;
    let current_dir = std::env::current_dir().unwrap();
    let cursor = config.cursor.unwrap_or_default();
    let breadcrumbs = cursor.initial_load()?;

    let ctx = Context {
        chain,
        intersect,
        finalize,
        current_dir,
        breadcrumbs,
    };

    let source = config.source.bootstrapper(&ctx)?;

    let filters = config
        .filters
        .into_iter()
        .flatten()
        .map(|x| x.bootstrapper(&ctx))
        .collect::<Result<_, _>>()?;

    let sink = config.sink.bootstrapper(&ctx)?;

    let cursor = cursor.bootstrapper(&ctx)?;

    let retries = define_gasket_policy(config.retries.as_ref());

    let daemon = connect_stages(source, filters, sink, cursor, retries)?;

    info!("oura is running");

    let daemon = Arc::new(daemon);

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let prometheus = tokio_rt.spawn(serve_prometheus(daemon.clone(), config.metrics));
    let tui = tokio_rt.spawn(console::render(daemon.clone(), args.tui));

    daemon.block();

    info!("oura is stopping");

    daemon.teardown();
    prometheus.abort();
    tui.abort();

    Ok(())
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// config file to load by the daemon
    #[clap(long, value_parser)]
    config: Option<std::path::PathBuf>,

    /// display the terminal UI
    #[clap(long, action)]
    tui: bool,
}
