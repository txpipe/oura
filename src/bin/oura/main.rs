mod daemon;
mod watch;

use std::process;

use clap::{App, AppSettings, Arg, SubCommand};

type Error = oura::framework::Error;

fn main() {
    let args = App::new("app")
        .name("oura")
        .about("the tail of cardano")
        .subcommand(
            SubCommand::with_name("watch")
                .arg(Arg::with_name("socket").required(true))
                .arg(
                    Arg::with_name("bearer")
                        .long("bearer")
                        .takes_value(true)
                        .possible_values(&["tcp", "unix"]),
                )
                .arg(Arg::with_name("magic").long("magic").takes_value(true))
                .arg(
                    Arg::with_name("since")
                        .long("since")
                        .takes_value(true)
                        .help(
                        "point in the chain to start reading from, expects format `slot,hex-hash`",
                    ),
                )
                .arg(
                    Arg::with_name("mode")
                        .long("mode")
                        .takes_value(true)
                        .possible_values(&["node", "client"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("daemon").arg(
                Arg::with_name("config")
                    .long("config")
                    .takes_value(true)
                    .help("config file to load by the daemon"),
            ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let result = match args.subcommand() {
        ("watch", Some(args)) => watch::run(args),
        ("daemon", Some(args)) => daemon::run(args),
        _ => Err("nothing to do".into()),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {:#?}", err);
        process::exit(1);
    }

    process::exit(0);
}
