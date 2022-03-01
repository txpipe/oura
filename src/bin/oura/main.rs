mod daemon;
mod dump;
mod watch;

use std::process;

use clap::{Arg, Command};

type Error = oura::Error;

fn main() {
    let args = Command::new("app")
        .name("oura")
        .about("the tail of cardano")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("watch")
                .arg(Arg::new("socket").required(true))
                .arg(
                    Arg::new("bearer")
                        .long("bearer")
                        .takes_value(true)
                        .possible_values(&["tcp", "unix"]),
                )
                .arg(Arg::new("magic").long("magic").takes_value(true))
                .arg(Arg::new("since").long("since").takes_value(true).help(
                    "point in the chain to start reading from, expects format `slot,hex-hash`",
                ))
                .arg(
                    Arg::new("throttle")
                        .long("throttle")
                        .takes_value(true)
                        .help("milliseconds to wait between output lines (for easier reading)"),
                )
                .arg(
                    Arg::new("mode")
                        .long("mode")
                        .takes_value(true)
                        .possible_values(&["node", "client"]),
                ),
        )
        .subcommand(
            Command::new("dump")
                .arg(Arg::new("socket").required(true))
                .arg(
                    Arg::new("bearer")
                        .long("bearer")
                        .takes_value(true)
                        .possible_values(&["tcp", "unix"]),
                )
                .arg(Arg::new("magic").long("magic").takes_value(true))
                .arg(Arg::new("since").long("since").takes_value(true).help(
                    "point in the chain to start reading from, expects format `slot,hex-hash`",
                ))
                .arg(
                    Arg::new("mode")
                        .long("mode")
                        .takes_value(true)
                        .possible_values(&["node", "client"]),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .takes_value(true)
                        .help("path-like prefix for the log files (fallbacks to stdout output)"),
                ),
        )
        .subcommand(
            Command::new("daemon").arg(
                Arg::new("config")
                    .long("config")
                    .takes_value(true)
                    .help("config file to load by the daemon"),
            ),
        )
        .arg_required_else_help(true)
        .get_matches();

    let result = match args.subcommand() {
        Some(("watch", args)) => watch::run(args),
        Some(("dump", args)) => dump::run(args),
        Some(("daemon", args)) => daemon::run(args),
        _ => Err("nothing to do".into()),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {:#?}", err);
        process::exit(1);
    }

    process::exit(0);
}
