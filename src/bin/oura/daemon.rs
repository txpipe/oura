use clap;
use gasket::runtime::Tether;
use oura::{filters, framework::*, mappers, sinks, sources};
use pallas::{ledger::traverse::wellknown::GenesisValues, network::upstream::cursor::Cursor};
use serde::Deserialize;
use std::time::Duration;

use crate::console;

#[derive(Deserialize)]
struct ConfigRoot {
    source: sources::Config,
    filter: Option<filters::Config>,
    mapper: Option<mappers::Config>,
    sink: sinks::Config,
    intersect: IntersectConfig,
    finalize: Option<FinalizeConfig>,
    chain: Option<GenesisValues>,
    policy: Option<RuntimePolicy>,
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
    source: Vec<Tether>,
    filter: Vec<Tether>,
    mapper: Vec<Tether>,
    sink: Vec<Tether>,
}

impl Runtime {
    fn all_tethers(&self) -> impl Iterator<Item = &Tether> {
        self.source
            .iter()
            .chain(self.filter.iter())
            .chain(self.mapper.iter())
            .chain(self.sink.iter())
    }

    fn should_stop(&self) -> bool {
        self.all_tethers().any(|tether| match tether.check_state() {
            gasket::runtime::TetherState::Alive(x) => match x {
                gasket::runtime::StageState::StandBy => true,
                _ => false,
            },
            _ => true,
        })
    }

    fn shutdown(&self) {
        for tether in self.all_tethers() {
            let state = tether.check_state();
            log::warn!("dismissing stage: {} with state {:?}", tether.name(), state);
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

fn bootstrap(
    mut source: sources::Bootstrapper,
    mut filter: filters::Bootstrapper,
    mut mapper: mappers::Bootstrapper,
    mut sink: sinks::Bootstrapper,
) -> Result<Runtime, Error> {
    let (to_filter, from_source) = gasket::messaging::crossbeam::channel(100);
    source.connect_output(to_filter);
    filter.connect_input(from_source);

    let (to_mapper, from_filter) = gasket::messaging::crossbeam::channel(100);
    filter.connect_output(to_mapper);
    mapper.connect_input(from_filter);

    let (to_sink, from_mapper) = gasket::messaging::crossbeam::channel(100);
    mapper.connect_output(to_sink);
    sink.connect_input(from_mapper);

    let runtime = Runtime {
        source: source.spawn()?,
        filter: filter.spawn()?,
        mapper: mapper.spawn()?,
        sink: sink.spawn()?,
    };

    Ok(runtime)
}

pub fn run(args: &Args) -> Result<(), Error> {
    console::initialize(&args.console);

    let config = ConfigRoot::new(&args.config).map_err(|err| Error::config(err))?;

    let chain = config.chain.unwrap_or_default().into();
    let cursor = Cursor::new(config.intersect.into());
    let error_policy = config.policy.unwrap_or_default().into();
    let finalize = config.finalize;
    let current_dir = std::env::current_dir().unwrap();

    let ctx = Context {
        chain,
        error_policy,
        finalize,
        cursor,
        current_dir,
    };

    let source = config.source.bootstrapper(&ctx)?;
    let filter = config.filter.unwrap_or_default().bootstrapper(&ctx)?;
    let mapper = config.mapper.unwrap_or_default().bootstrapper(&ctx)?;
    let sink = config.sink.bootstrapper(&ctx)?;

    let runtime = bootstrap(source, filter, mapper, sink)?;

    log::info!("oura is running...");

    while !runtime.should_stop() {
        console::refresh(&args.console, runtime.all_tethers());
        std::thread::sleep(Duration::from_millis(1500));
    }

    log::info!("Oura is stopping...");
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
