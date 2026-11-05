use clap::Parser;
use std::process;

mod console;
mod dump;
mod run_daemon;
mod watch;

#[derive(Parser)]
#[clap(name = "Oura")]
#[clap(bin_name = "oura")]
#[clap(author, version, about, long_about = None)]
enum Oura {
    Daemon(run_daemon::Args),
    Watch(watch::Args),
    Dump(dump::Args),
}

fn main() {
    // Install a process-default rustls CryptoProvider before anything uses TLS.
    // In builds that pull more than one provider (e.g. `aws` brings in aws-lc-rs
    // alongside ring), rustls has no default and panics on the first handshake;
    // this guards every TLS-using feature (u5c, gcp, elasticsearch, hydra, …).
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args = Oura::parse();

    let result = match args {
        Oura::Daemon(x) => run_daemon::run(&x),
        Oura::Watch(x) => watch::run(&x),
        Oura::Dump(x) => dump::run(&x),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {err:#?}");
        process::exit(1);
    }

    process::exit(0);
}
