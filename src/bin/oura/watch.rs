use std::{str::FromStr, sync::Arc};

use clap::ArgMatches;
use oura::{
    mapper::Config as MapperConfig,
    pipelining::{SinkProvider, SourceProvider},
    sources::{AddressArg, BearerKind, IntersectArg, MagicArg},
    utils::{ChainWellKnownInfo, Utils, WithUtils},
};

use serde::Deserialize;

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
        true => args.value_of_t("magic")?,
        false => MagicArg::default(),
    };

    let intersect = match args.is_present("since") {
        true => Some(IntersectArg::Point(args.value_of_t("since")?)),
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

    let throttle = match args.is_present("throttle") {
        true => Some(args.value_of_t("throttle")?),
        false => None,
    };

    let mapper = MapperConfig {
        include_block_end_events: true,
        ..Default::default()
    };

    let well_known = ChainWellKnownInfo::try_from_magic(*magic)?;

    let utils = Arc::new(Utils::new(well_known));

    #[allow(deprecated)]
    let source_setup = match mode {
        PeerMode::AsNode => WatchSource::N2N(N2NConfig {
            address: AddressArg(bearer, socket),
            magic: Some(magic),
            well_known: None,
            min_depth: 0,
            mapper,
            since: None,
            intersect,
            retry_policy: None,
            finalize: None,
        }),
        PeerMode::AsClient => WatchSource::N2C(N2CConfig {
            address: AddressArg(bearer, socket),
            magic: Some(magic),
            well_known: None,
            min_depth: 0,
            mapper,
            since: None,
            intersect,
            retry_policy: None,
            finalize: None,
        }),
    };

    let sink_setup = oura::sinks::terminal::Config {
        throttle_min_span_millis: throttle,
    };

    let (source_handle, source_output) = match source_setup {
        WatchSource::N2C(c) => WithUtils::new(c, utils.clone()).bootstrap()?,
        WatchSource::N2N(c) => WithUtils::new(c, utils.clone()).bootstrap()?,
    };

    let sink_handle = WithUtils::new(sink_setup, utils).bootstrap(source_output)?;

    sink_handle.join().map_err(|_| "error in sink thread")?;
    source_handle.join().map_err(|_| "error in source thread")?;

    Ok(())
}

/// Creates the clap definition for this sub-command
pub(crate) fn command_definition<'a>() -> clap::Command<'a> {
    clap::Command::new("watch")
        .arg(clap::Arg::new("socket").required(true))
        .arg(
            clap::Arg::new("bearer")
                .long("bearer")
                .takes_value(true)
                .possible_values(&["tcp", "unix"]),
        )
        .arg(clap::Arg::new("magic").long("magic").takes_value(true))
        .arg(
            clap::Arg::new("since")
                .long("since")
                .takes_value(true)
                .help("point in the chain to start reading from, expects format `slot,hex-hash`"),
        )
        .arg(
            clap::Arg::new("throttle")
                .long("throttle")
                .takes_value(true)
                .help("milliseconds to wait between output lines (for easier reading)"),
        )
        .arg(
            clap::Arg::new("mode")
                .long("mode")
                .takes_value(true)
                .possible_values(&["node", "client"]),
        )
}
