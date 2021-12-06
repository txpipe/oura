use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};
use oura::{
    framework::*,
    sources::chain::{AddressArg, BearerKind, MagicArg, PeerMode},
};

type Error = Box<dyn std::error::Error>;

fn run_log(args: &ArgMatches) -> Result<(), Error> {
    let socket = value_t!(args, "socket", String)?;

    let bearer = match args.is_present("bearer") {
        true => value_t!(args, "bearer", BearerKind)?,
        false => BearerKind::Unix,
    };

    let source_setup = oura::sources::chain::Config {
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

fn run_daemon(_args: &ArgMatches) -> Result<(), Error> {
    todo!();
}

fn main() {
    //env_logger::init();

    let args = App::new("app")
        .name("oura")
        .about("the tail of cardano")
        .subcommand(
            SubCommand::with_name("log")
                .arg(Arg::with_name("socket").required(true))
                .arg(
                    Arg::with_name("bearer")
                        .long("bearer")
                        .takes_value(true)
                        .possible_values(&["tcp", "unix"]),
                )
                .arg(Arg::with_name("magic").long("magic").takes_value(true))
                .arg(
                    Arg::with_name("mode")
                        .long("mode")
                        .takes_value(true)
                        .possible_values(&["node", "client"]),
                ),
        )
        .subcommand(SubCommand::with_name("daemon"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match args.subcommand() {
        ("log", Some(args)) => run_log(args).unwrap(),
        ("daemon", Some(args)) => run_daemon(args).unwrap(),
        _ => (),
    }
}
