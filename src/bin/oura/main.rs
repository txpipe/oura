use clap::Parser;
use std::process;

mod console;
mod daemon;
mod prometheus;

#[derive(Parser)]
#[clap(name = "Oura")]
#[clap(bin_name = "oura")]
#[clap(author, version, about, long_about = None)]
enum Oura {
    Daemon(daemon::Args),
}

fn main() {
    let args = Oura::parse();

    let result = match args {
        Oura::Daemon(x) => daemon::run(&x),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {err:#?}");
        process::exit(1);
    }

    process::exit(0);
}
