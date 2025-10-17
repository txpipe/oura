use oura::watch::cardano;

#[cfg(feature = "btc")]
use oura::watch::btc;

#[cfg(feature = "eth")]
use oura::watch::eth;

#[cfg(feature = "substrate")]
use oura::watch::substrate;

use clap::{Parser, Subcommand};
use oura::framework::Error;

pub fn run(args: &Args) -> Result<(), Error> {
    match &args.command {
        WatchCommand::Cardano(cardano_args) => cardano::run(cardano_args),

        #[cfg(feature = "btc")]
        WatchCommand::Bitcoin(btc_args) => btc::run(btc_args),
        
        #[cfg(feature = "eth")]
        WatchCommand::Ethereum(eth_args) => eth::run(eth_args),

        #[cfg(feature = "substrate")]
        WatchCommand::Substrate(substrate_args) => substrate::run(substrate_args),
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

    /// Watch Ethereum blockchain
    #[cfg(feature = "eth")]
    Ethereum(eth::Args),

    /// Watch Substrate blockchain
    #[cfg(feature = "substrate")]
    Substrate(substrate::Args),
}
