use clap::{value_t, ArgMatches};
use oura::{
    framework::*,
    sources::n2c::{AddressArg, BearerKind, MagicArg, PeerMode},
};

pub fn run(args: &ArgMatches) -> Result<(), Error> {
    let socket = value_t!(args, "socket", String)?;

    let bearer = match args.is_present("bearer") {
        true => value_t!(args, "bearer", BearerKind)?,
        false => BearerKind::Unix,
    };

    let source_setup = oura::sources::n2c::Config {
        address: AddressArg(bearer, socket),
        magic: match args.is_present("magic") {
            true => Some(value_t!(args, "magic", MagicArg)?),
            false => None,
        },
        mode: match args.is_present("mode") {
            true => Some(value_t!(args, "mode", PeerMode)?),
            false => None,
        },
    };

    let sink_setup = oura::sinks::terminal::Config::default();

    let (tx, rx) = std::sync::mpsc::channel();

    let source = source_setup.bootstrap(tx)?;
    let sink = sink_setup.bootstrap(rx)?;

    sink.join().map_err(|_| "error in sink thread")?;
    source.join().map_err(|_| "error in source thread")?;

    Ok(())
}
