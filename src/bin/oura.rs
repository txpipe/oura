use clap::{value_t, App, Arg, ArgMatches, SubCommand};
use oura::sources::chain::{AddressArg, BearerKind, MagicArg, PeerMode};

type Error = Box<dyn std::error::Error>;

fn run_log(args: &ArgMatches) -> Result<(), Error> {
    let address = value_t!(args, "node", String)?;

    let bearer = match args.is_present("bearer") {
        true => value_t!(args, "bearer", BearerKind)?,
        false => BearerKind::Unix,
    };

    let node = AddressArg(bearer, address);

    let mode = match args.is_present("mode") {
        true => Some(value_t!(args, "mode", PeerMode)?),
        false => None,
    };

    let magic = match args.is_present("magic") {
        true => Some(value_t!(args, "magic", MagicArg)?),
        false => None,
    };

    let (tx, rx) = std::sync::mpsc::channel();

    let source = oura::sources::chain::bootstrap(node, magic, mode, tx).unwrap();
    let sink = oura::sinks::terminal::bootstrap(rx).unwrap();

    sink.join().map_err(|_| "error in sink thread")?;
    source.join().map_err(|_| "error in source thread")?;

    Ok(())
}

fn main() {
    //env_logger::init();

    let args = App::new("app")
        .name("oura")
        .about("the tail of cardano")
        .subcommand(
            SubCommand::with_name("log")
                .arg(
                    Arg::with_name("node")
                        .long("node")
                        .takes_value(true)
                        .required(true),
                )
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
        .get_matches();

    match args.subcommand() {
        ("log", Some(args)) => run_log(args).unwrap(),
        _ => unreachable!(),
    }
}
