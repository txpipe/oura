
use gasket::daemon::Daemon;
use crate::{cursor, filters, framework::*, sinks, sources};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub address: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigRoot {
    pub source: sources::Config,
    pub filters: Option<Vec<filters::Config>>,
    pub sink: sinks::Config,
    pub intersect: IntersectConfig,
    pub finalize: Option<FinalizeConfig>,
    pub chain: Option<ChainConfig>,
    pub retries: Option<gasket::retries::Policy>,
    pub cursor: Option<cursor::Config>,
    pub metrics: Option<MetricsConfig>,
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

pub fn run_daemon(config: ConfigRoot) -> Result<Daemon, Error> {
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
    Ok(daemon)
}

