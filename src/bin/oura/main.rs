use clap::Parser;
use std::process;

mod console;
mod dump;
mod run_daemon;

#[derive(Parser)]
#[clap(name = "Oura")]
#[clap(bin_name = "oura")]
#[clap(author, version, about, long_about = None)]
enum Oura {
    Daemon(run_daemon::Args),
    Dump(dump::Args),
}

fn main() {
    let args = Oura::parse();

    let result = match args {
        Oura::Daemon(x) => run_daemon::run(&x),
        Oura::Dump(x) => dump::run(&x),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {err:#?}");
        process::exit(1);
    }

    process::exit(0);
}
