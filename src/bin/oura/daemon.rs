use gasket::runtime::Tether;
use oura::{cursor, filters, framework::*, sinks, sources};
use serde::Deserialize;
use std::{fmt::Debug, sync::Arc, time::Duration};
use tracing::{info, warn};

use crate::{console, prometheus};

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
    metrics: Option<prometheus::Config>,
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

pub struct Runtime {
    pub source: Tether,
    pub filters: Vec<Tether>,
    pub sink: Tether,
    pub cursor: Tether,
}

impl Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").finish()
    }
}

impl Runtime {
    pub fn all_tethers(&self) -> impl Iterator<Item = &Tether> {
        std::iter::once(&self.source)
            .chain(self.filters.iter())
            .chain(std::iter::once(&self.sink))
            .chain(std::iter::once(&self.cursor))
    }

    fn should_stop(&self) -> bool {
        self.all_tethers().any(|tether| match tether.check_state() {
            gasket::runtime::TetherState::Alive(x) => {
                matches!(x, gasket::runtime::StagePhase::Ended)
            }
            _ => true,
        })
    }

    fn shutdown(&self) {
        for tether in self.all_tethers() {
            let state = tether.check_state();
            warn!("dismissing stage: {} with state {:?}", tether.name(), state);
            tether.dismiss_stage().expect("stage stops");

            // Can't join the stage because there's a risk of deadlock, usually
            // because a stage gets stuck sending into a port which depends on a
            // different stage not yet dismissed. The solution is to either
            // create a DAG of dependencies and dismiss in the
            // correct order, or implement a 2-phase teardown where
            // ports are disconnected and flushed before joining the
            // stage.

            //tether.join_stage();
        }
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

fn bootstrap(
    mut source: sources::Bootstrapper,
    mut filters: Vec<filters::Bootstrapper>,
    mut sink: sinks::Bootstrapper,
    mut cursor: cursor::Bootstrapper,
    policy: gasket::runtime::Policy,
) -> Result<Runtime, Error> {
    let mut prev = source.borrow_output();

    for filter in filters.iter_mut() {
        gasket::messaging::tokio::connect_ports(prev, filter.borrow_input(), 100);
        prev = filter.borrow_output();
    }

    gasket::messaging::tokio::connect_ports(prev, sink.borrow_input(), 100);
    let prev = sink.borrow_cursor();

    gasket::messaging::tokio::connect_ports(prev, cursor.borrow_track(), 100);

    let runtime = Runtime {
        source: source.spawn(policy.clone()),
        filters: filters
            .into_iter()
            .map(|x| x.spawn(policy.clone()))
            .collect(),
        sink: sink.spawn(policy.clone()),
        cursor: cursor.spawn(policy),
    };

    Ok(runtime)
}

async fn monitor_loop(
    runtime: &Arc<Runtime>,
    metrics: Option<prometheus::Config>,
    console: Option<super::console::Mode>,
) {
    if let Some(metrics) = metrics {
        info!("starting metrics exporter");
        let runtime = runtime.clone();
        tokio::spawn(async { prometheus::initialize(metrics, runtime).await });
    }

    while !runtime.should_stop() {
        // TODO: move console refresh to it's own tokio thread
        console::refresh(&console, runtime.all_tethers());
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
}

pub fn run(args: &Args) -> Result<(), Error> {
    console::initialize(&args.console);

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
    let runtime = bootstrap(source, filters, sink, cursor, retries)?;
    let runtime = Arc::new(runtime);

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    info!("oura is running...");

    tokio_rt.block_on(async { monitor_loop(&runtime, config.metrics, args.console.clone()).await });

    info!("Oura is stopping...");

    runtime.shutdown();

    Ok(())
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, value_parser)]
    //#[clap(description = "config file to load by the daemon")]
    config: Option<std::path::PathBuf>,

    #[clap(long, value_parser)]
    //#[clap(description = "type of progress to display")],
    console: Option<console::Mode>,
}
