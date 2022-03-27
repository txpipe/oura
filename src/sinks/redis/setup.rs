//use std::time::Duration;
use std::str::FromStr;
use redis::{Client};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
    model::Event,
};

use super::run::*;

#[derive(Debug, Deserialize, Clone)]
pub enum PartitionStrategy {
    ByBlock,
    ByTx,
    Random,
}

impl PartitionStrategy {

    pub fn define_key(event: &Event, strategy: &PartitionStrategy) -> Option<String> {
        match strategy {
            PartitionStrategy::ByBlock => event.context.block_number.map(|n| n.to_string()),
            PartitionStrategy::ByTx    => {
                if let Some (txhash) = event.context.tx_hash.clone() {
                    Some(txhash)
                } else {
                    None
                }
            },
            PartitionStrategy::Random  => None,
        }
    }
}

impl FromStr for PartitionStrategy {
    type Err = crate::Error;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        match src {
            "block"       => Ok(PartitionStrategy::ByBlock),
            "tx"          => Ok(PartitionStrategy::ByTx),
            "rnd"         => Ok(PartitionStrategy::Random),
            _             => Err(format!("PartitionStrategy '{}' not supported",src).into()),
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub redis_server      : String,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        // redis://[<username>][:<password>@]<hostname>[:port][/<db>]
        let client = Client::open(self.inner.redis_server.clone())?;
        let mut connection = client.get_connection()?;
        println!("Connected to Redis Database!");

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            producer_loop(input, utils, &mut connection).expect("terminal sink loop failed");
        });

        Ok(handle)
    }
}
