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
    // The U5C source connects over TLS via tonic/rustls. In builds that pull
    // more than one rustls crypto provider (e.g. `aws` brings in aws-lc-rs
    // alongside ring), rustls has no process-default provider and panics on the
    // first TLS handshake. Install one explicitly before anything uses TLS.
    #[cfg(feature = "u5c")]
    {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

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
