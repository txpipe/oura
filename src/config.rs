use std::thread::JoinHandle;

use pallas::ouroboros::network::handshake::{MAINNET_MAGIC, TESTNET_MAGIC};
use serde_derive::{Deserialize, Serialize};

use crate::{pipelining::StageReceiver, Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
}

impl ChainWellKnownInfo {
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_known_slot: 4492799,
                shelley_known_hash:
                    "f8084c61b6a238acec985b59310b6ecec49c0ab8352249afd7268da5cff2a457".to_string(),
                shelley_known_time: 1596059071,
            }),
            TESTNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_known_slot: 1598399,
                shelley_known_hash:
                    "7e16781b40ebf8b6da18f7b5e8ade855d6738095ef2f1c58c77e88b6e45997a4".to_string(),
                shelley_known_time: 1595967596,
            }),
            _ => Err("can't infer well-known chain infro from specified magic".into()),
        }
    }
}
