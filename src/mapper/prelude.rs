use crate::{
    framework::{Event, EventContext, EventData},
    pipelining::StageSender,
};
use merge::Merge;
use serde_derive::Deserialize;

use pallas::ouroboros::network::handshake::{MAINNET_MAGIC, TESTNET_MAGIC};
use serde_derive::Serialize;

use crate::Error;

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

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub include_block_end_events: bool,

    #[serde(default)]
    pub include_transaction_details: bool,
    
    #[serde(default)]
    pub include_transaction_end_events: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct EventWriter {
    context: EventContext,
    output: StageSender,
    chain_info: Option<ChainWellKnownInfo>,
    pub(crate) config: Config,
}

impl EventWriter {
    pub fn new(
        output: StageSender,
        chain_info: Option<ChainWellKnownInfo>,
        config: Config,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            chain_info,
            config,
        }
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        self.output.send(evt)?;

        Ok(())
    }

    pub fn append_from<T>(&self, source: T) -> Result<(), Error>
    where
        T: Into<EventData>,
    {
        let evt = Event {
            context: self.context.clone(),
            data: source.into(),
            fingerprint: None,
        };

        self.output.send(evt)?;

        Ok(())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
            chain_info: self.chain_info.clone(),
            config: self.config.clone(),
        }
    }

    pub fn compute_timestamp(&self, slot: u64) -> Option<u64> {
        self.chain_info
            .as_ref()
            .map(|info| info.shelley_known_time + (slot - info.shelley_known_slot))
    }
}
