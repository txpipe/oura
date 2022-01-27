use serde::Deserialize;
use std::{str::FromStr, sync::Arc};

use clap::ArgMatches;

use oura::{
    mapper::Config as MapperConfig,
    pipelining::{BootstrapResult, SinkProvider, SourceProvider, StageReceiver},
    sources::{AddressArg, BearerKind, MagicArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use oura::sinks::stdout::Config as StdoutConfig;
use oura::sources::n2c::Config as N2CConfig;
use oura::sources::n2n::Config as N2NConfig;

#[cfg(feature = "logs")]
use oura::sinks::logs::Config as LogsConfig;

use crate::Error;

#[derive(Clone, Debug, Deserialize)]
pub enum PeerMode {
    AsNode,
    AsClient,
}

impl FromStr for PeerMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "node" => Ok(PeerMode::AsNode),
            "client" => Ok(PeerMode::AsClient),
            _ => Err("can't parse peer mode (valid values: client|node)"),
        }
    }
}

enum DumpSource {
    N2C(N2CConfig),
    N2N(N2NConfig),
}

enum DumpSink {
    Stdout(StdoutConfig),
    #[cfg(feature = "logs")]
    Logs(LogsConfig),
}

impl SinkProvider for DumpSink {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        match self {
            DumpSink::Stdout(c) => c.bootstrap(input),
            #[cfg(feature = "logs")]
            DumpSink::Logs(c) => c.bootstrap(input),
        }
    }
}

pub fn run(args: &ArgMatches) -> Result<(), Error> {
    env_logger::builder()
        .filter_module("oura::dump", log::LevelFilter::Info)
        .init();

    let socket = args.value_of_t("socket")?;

    let bearer = match args.is_present("bearer") {
        true => args.value_of_t("bearer")?,
        #[cfg(target_family = "unix")]
        false => BearerKind::Unix,
        #[cfg(target_family = "windows")]
        false => BearerKind::Tcp,
    };

    let magic = match args.is_present("magic") {
        true => args.value_of_t("magic")?,
        false => MagicArg::default(),
    };

    let since = match args.is_present("since") {
        true => Some(args.value_of_t("since")?),
        false => None,
    };

    let mode = match (args.is_present("mode"), &bearer) {
        (true, _) => args
            .value_of_t("mode")
            .expect("invalid value for 'mode' arg"),
        (false, BearerKind::Tcp) => PeerMode::AsNode,
        #[cfg(target_family = "unix")]
        (false, BearerKind::Unix) => PeerMode::AsClient,
    };

    let output: Option<String> = match args.is_present("output") {
        #[cfg(feature = "logs")]
        true => Some(args.value_of_t("output")?),
        _ => None,
    };

    let mapper = MapperConfig {
        include_block_end_events: true,
        ..Default::default()
    };

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)?;

    let utils = Arc::new(Utils::new(well_known));

    #[allow(deprecated)]
    let source_setup = match mode {
        PeerMode::AsNode => DumpSource::N2N(N2NConfig {
            address: AddressArg(bearer, socket),
            magic: Some(magic),
            well_known: None,
            mapper,
            since,
        }),
        PeerMode::AsClient => DumpSource::N2C(N2CConfig {
            address: AddressArg(bearer, socket),
            magic: Some(magic),
            well_known: None,
            mapper,
            since,
        }),
    };

    let sink_setup = match &output {
        #[cfg(feature = "logs")]
        Some(x) => DumpSink::Logs(LogsConfig {
            output_path: Some(x.to_owned()),
            ..Default::default()
        }),
        _ => DumpSink::Stdout(StdoutConfig {
            ..Default::default()
        }),
    };

    let (source_handle, source_output) = match source_setup {
        DumpSource::N2C(c) => WithUtils::new(c, utils).bootstrap()?,
        DumpSource::N2N(c) => WithUtils::new(c, utils).bootstrap()?,
    };

    let sink_handle = sink_setup.bootstrap(source_output)?;

    log::info!(
        "Oura started dumping events to {}",
        output.as_deref().unwrap_or("stdout")
    );

    sink_handle.join().map_err(|_| "error in sink thread")?;
    source_handle.join().map_err(|_| "error in source thread")?;

    Ok(())
}
