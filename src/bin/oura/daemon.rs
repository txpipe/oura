use gasket::runtime::Tether;
use oura::{filters, framework::*, sinks, sources};
use serde::Deserialize;
use std::{collections::VecDeque, time::Duration};
use tracing::{info, warn};

use crate::console;

#[derive(Deserialize)]
struct ConfigRoot {
    source: sources::Config,
    filters: Option<Vec<filters::Config>>,
    sink: sinks::Config,
    intersect: IntersectConfig,
    finalize: Option<FinalizeConfig>,
    chain: Option<ChainConfig>,
    retries: Option<gasket::retries::Policy>,
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

struct Runtime {
    source: Tether,
    filters: Vec<Tether>,
    sink: Tether,
}

impl Runtime {
    fn all_tethers(&self) -> impl Iterator<Item = &Tether> {
        std::iter::once(&self.source)
            .chain(self.filters.iter())
            .chain(std::iter::once(&self.sink))
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

fn chain_stages<'a>(
    source: &'a mut dyn StageBootstrapper,
    filters: Vec<&'a mut dyn StageBootstrapper>,
    sink: &'a mut dyn StageBootstrapper,
) {
    let mut prev = source;

    for filter in filters {
        let (to_next, from_prev) = gasket::messaging::tokio::channel(100);
        prev.connect_output(to_next);
        filter.connect_input(from_prev);
        prev = filter;
    }

    let (to_next, from_prev) = gasket::messaging::tokio::channel(100);
    prev.connect_output(to_next);
    sink.connect_input(from_prev);
}

fn bootstrap(
    mut source: sources::Bootstrapper,
    mut filters: Vec<filters::Bootstrapper>,
    mut sink: sinks::Bootstrapper,
    policy: gasket::runtime::Policy,
) -> Result<Runtime, Error> {
    chain_stages(
        &mut source,
        filters
            .iter_mut()
            .map(|x| x as &mut dyn StageBootstrapper)
            .collect::<Vec<_>>(),
        &mut sink,
    );

    let runtime = Runtime {
        source: source.spawn(policy.clone()),
        filters: filters
            .into_iter()
            .map(|x| x.spawn(policy.clone()))
            .collect(),
        sink: sink.spawn(policy),
    };

    Ok(runtime)
}

pub fn run(args: &Args) -> Result<(), Error> {
    console::initialize(&args.console);

    let config = ConfigRoot::new(&args.config).map_err(Error::config)?;

    let chain = config.chain.unwrap_or_default();
    let intersect = config.intersect;
    let finalize = config.finalize;
    let current_dir = std::env::current_dir().unwrap();

    // TODO: load from persistence mechanism
    let cursor = Cursor::new(VecDeque::new());

    let ctx = Context {
        chain,
        intersect,
        finalize,
        cursor,
        current_dir,
    };

    let source = config.source.bootstrapper(&ctx)?;

    let filters = config
        .filters
        .into_iter()
        .flatten()
        .map(|x| x.bootstrapper(&ctx))
        .collect::<Result<_, _>>()?;

    let sink = config.sink.bootstrapper(&ctx)?;

    let retries = define_gasket_policy(config.retries.as_ref());
    let runtime = bootstrap(source, filters, sink, retries)?;

    info!("oura is running...");

    while !runtime.should_stop() {
        console::refresh(&args.console, runtime.all_tethers());
        std::thread::sleep(Duration::from_millis(1500));
    }

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
