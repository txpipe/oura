use oura::watch::cardano;

#[cfg(feature = "btc")]
use oura::watch::btc;

use clap::{Parser, Subcommand};
use oura::framework::Error;

pub fn run(args: &Args) -> Result<(), Error> {
    match &args.command {
        WatchCommand::Cardano(cardano_args) => cardano::run(cardano_args),

        #[cfg(feature = "btc")]
        WatchCommand::Bitcoin(btc_args) => btc::run(btc_args),
    }
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: WatchCommand,
}

#[derive(Subcommand, Clone)]
pub enum WatchCommand {
    /// Watch Cardano blockchain
    Cardano(cardano::Args),

    /// Watch Bitcoin blockchain
    #[cfg(feature = "btc")]
    Bitcoin(btc::Args),
}
