mod daemon;
mod dump;
mod watch;

use std::process;

use clap::Command;

type Error = oura::Error;

fn main() {
    let args = Command::new("app")
        .name("oura")
        .about("the tail of cardano")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(watch::command_definition())
        .subcommand(dump::command_definition())
        .subcommand(daemon::command_definition())
        .arg_required_else_help(true)
        .get_matches();

    let result = match args.subcommand() {
        Some(("watch", args)) => watch::run(args),
        Some(("dump", args)) => dump::run(args),
        Some(("daemon", args)) => daemon::run(args),
        _ => Err("nothing to do".into()),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {err:#?}");
        process::exit(1);
    }

    process::exit(0);
}
