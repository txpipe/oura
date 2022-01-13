use std::str::FromStr;

use clap::ArgMatches;
use oura::{
    mapper::Config as MapperConfig,
    sources::{AddressArg, BearerKind}, pipelining::{SourceConfig, PartialBootstrapResult, SinkConfig},
};

use serde_derive::Deserialize;

use oura::sources::n2c::Config as N2CConfig;
use oura::sources::n2n::Config as N2NConfig;

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

enum WatchSource {
    N2C(N2CConfig),
    N2N(N2NConfig),
}

impl SourceConfig for WatchSource {
    fn bootstrap(&self) -> PartialBootstrapResult {
        match self {
            WatchSource::N2C(c) => c.bootstrap(),
            WatchSource::N2N(c) => c.bootstrap(),
        }
    }
}

pub fn run(args: &ArgMatches) -> Result<(), Error> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Error)
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
        true => Some(args.value_of_t("magic")?),
        false => None,
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

    let source_setup = match mode {
        PeerMode::AsNode => WatchSource::N2N(N2NConfig {
            address: AddressArg(bearer, socket),
            magic,
            well_known: None,
            mapper: MapperConfig::default(),
            since,
        }),
        PeerMode::AsClient => WatchSource::N2C(N2CConfig {
            address: AddressArg(bearer, socket),
            magic,
            well_known: None,
            mapper: MapperConfig::default(),
            since,
        }),
    };

    let sink_setup = oura::sinks::terminal::Config::default();

    let (source_handle, source_output) = source_setup.bootstrap()?;
    let sink_handle = sink_setup.bootstrap(source_output)?;

    sink_handle.join().map_err(|_| "error in sink thread")?;
    source_handle.join().map_err(|_| "error in source thread")?;

    Ok(())
}
