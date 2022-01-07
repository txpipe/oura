mod daemon;
mod watch;

use std::process;

use clap::{App, AppSettings, Arg};

type Error = oura::framework::Error;

fn main() {
    let args = App::new("app")
        .name("oura")
        .about("the tail of cardano")
        .version("v0.3.9")
        .subcommand(
            App::new("watch")
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
                ),
        )
        .subcommand(
            App::new("daemon").arg(
                Arg::new("config")
                    .long("config")
                    .takes_value(true)
                    .help("config file to load by the daemon"),
            ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let result = match args.subcommand() {
        Some(("watch", args)) => watch::run(args),
        Some(("daemon", args)) => daemon::run(args),
        _ => Err("nothing to do".into()),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {:#?}", err);
        process::exit(1);
    }

    process::exit(0);
}
