use clap;
use oura::{bootstrap, crosscut, filter, mapper, sinks, sources};
use serde::Deserialize;
use std::time::Duration;

use crate::console;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ChainConfig {
    Mainnet,
    Testnet,
    PreProd,
    Preview,
    Custom(crosscut::ChainWellKnownInfo),
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl From<ChainConfig> for crosscut::ChainWellKnownInfo {
    fn from(other: ChainConfig) -> Self {
        match other {
            ChainConfig::Mainnet => crosscut::ChainWellKnownInfo::mainnet(),
            ChainConfig::Testnet => crosscut::ChainWellKnownInfo::testnet(),
            ChainConfig::PreProd => crosscut::ChainWellKnownInfo::preprod(),
            ChainConfig::Preview => crosscut::ChainWellKnownInfo::preview(),
            ChainConfig::Custom(x) => x,
        }
    }
}

#[derive(Deserialize)]
struct ConfigRoot {
    source: sources::Config,
    filter: Option<filter::Config>,
    mapper: Vec<mapper::Config>,
    sink: sinks::Config,
    intersect: crosscut::IntersectConfig,
    finalize: Option<crosscut::FinalizeConfig>,
    chain: Option<ChainConfig>,
    policy: Option<crosscut::policies::RuntimePolicy>,
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

fn should_stop(pipeline: &bootstrap::Pipeline) -> bool {
    pipeline
        .tethers
        .iter()
        .any(|tether| match tether.check_state() {
            gasket::runtime::TetherState::Alive(x) => match x {
                gasket::runtime::StageState::StandBy => true,
                _ => false,
            },
            _ => true,
        })
}

fn shutdown(pipeline: bootstrap::Pipeline) {
    for tether in pipeline.tethers {
        let state = tether.check_state();
        log::warn!("dismissing stage: {} with state {:?}", tether.name(), state);
        tether.dismiss_stage().expect("stage stops");

        // Can't join the stage because there's a risk of deadlock, usually
        // because a stage gets stuck sending into a port which depends on a
        // different stage not yet dismissed. The solution is to either create a
        // DAG of dependencies and dismiss in the correct order, or implement a
        // 2-phase teardown where ports are disconnected and flushed
        // before joining the stage.

        //tether.join_stage();
    }
}

pub fn run(args: &Args) -> Result<(), oura::Error> {
    console::initialize(&args.console);

    let config = ConfigRoot::new(&args.config)
        .map_err(|err| oura::Error::ConfigError(format!("{:?}", err)))?;

    let chain = config.chain.unwrap_or_default().into();
    let policy = config.policy.unwrap_or_default().into();

    let source = config
        .source
        .bootstrapper(&chain, &config.intersect, &config.finalize, &policy);

    let filter = config.filter.unwrap_or_default().bootstrapper(&policy);

    let mapper = config.mapper.unwrap_or_default().bootstrapper(&policy);

    let sink = config.sink.plugin(&chain, &config.intersect, &policy);

    let pipeline = bootstrap::build(source, filter, mapper, sink)?;

    log::info!("oura is running...");

    while !should_stop(&pipeline) {
        console::refresh(&args.console, &pipeline);
        std::thread::sleep(Duration::from_millis(1500));
    }

    log::info!("Oura is stopping...");

    shutdown(pipeline);

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
