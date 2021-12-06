mod daemon;
mod watch;

use clap::{App, AppSettings, Arg, SubCommand};

type Error = Box<dyn std::error::Error>;

fn main() {
    //env_logger::init();

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

    match args.subcommand() {
        ("watch", Some(args)) => watch::run(args).unwrap(),
        ("daemon", Some(args)) => daemon::run(args).unwrap(),
        _ => (),
    }
}
