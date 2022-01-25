use crate::{
    framework::{Event, EventContext, EventData},
    pipelining::StageSender,
    utils::time::{NaiveConfig as TimeConfig, NaiveProvider as NaiveTime, TimeProvider},
};
use merge::Merge;
use serde::Deserialize;

use pallas::ouroboros::network::{
    handshake::{MAINNET_MAGIC, TESTNET_MAGIC},
    machines::primitives::Point,
};
use serde::Serialize;

use crate::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub shelley_slot_length: u32,
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
}

impl ChainWellKnownInfo {
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_slot_length: 1,
                shelley_known_slot: 4492800,
                shelley_known_hash:
                    "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de".to_string(),
                shelley_known_time: 1596059091,
            }),
            TESTNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_slot_length: 1,
                shelley_known_slot: 1598400,
                shelley_known_hash:
                    "02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f".to_string(),
                shelley_known_time: 1595967616,
            }),
            _ => Err("can't infer well-known chain infro from specified magic".into()),
        }
    }
}

// HACK: to glue together legacy config with new time provider
impl From<ChainWellKnownInfo> for TimeConfig {
    fn from(other: ChainWellKnownInfo) -> Self {
        TimeConfig {
            slot_length: other.shelley_slot_length,
            start_slot: other.shelley_known_slot,
            start_timestamp: other.shelley_known_time,
        }
    }
}

impl TryFrom<ChainWellKnownInfo> for Point {
    type Error = crate::Error;

    fn try_from(other: ChainWellKnownInfo) -> Result<Self, Self::Error> {
        let out = Point(
            other.shelley_known_slot,
            hex::decode(other.shelley_known_hash)?,
        );

        Ok(out)
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

#[derive(Clone)]
pub(crate) struct EventWriter {
    context: EventContext,
    output: StageSender,
    time_provider: Option<NaiveTime>,
    pub(crate) config: Config,
}

impl EventWriter {
    pub fn new(
        output: StageSender,
        well_known: Option<ChainWellKnownInfo>,
        config: Config,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            time_provider: well_known.map(|x| NaiveTime::new(x.into())),
            config,
        }
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        self.output
            .send(evt)
            .expect("error sending event through output stage, pipeline must have crashed.");

        Ok(())
    }

    pub fn append_from<T>(&self, source: T) -> Result<(), Error>
    where
        T: Into<EventData>,
    {
        self.append(source.into())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
            time_provider: self.time_provider.clone(),
            config: self.config.clone(),
        }
    }

    pub fn compute_timestamp(&self, slot: u64) -> Option<u64> {
        match &self.time_provider {
            Some(provider) => provider.slot_to_wallclock(slot).ok(),
            _ => None,
        }
    }
}
