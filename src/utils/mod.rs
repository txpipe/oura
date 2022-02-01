//! Pipeline-wide utilities
//!
//! This module includes general-purpose utilities that could potentially be
//! used by more than a single stage. The entry point to this utilities is
//! desgined as singelton [`Utils`] instance shared by all stages through an Arc
//! pointer.

use std::sync::Arc;

use pallas::ouroboros::network::{
    handshake::{MAINNET_MAGIC, TESTNET_MAGIC},
    machines::primitives::Point,
};

use serde::{Deserialize, Serialize};

use crate::{
    model::Event,
    utils::{
        bech32::{Bech32Config, Bech32Provider},
        time::{NaiveConfig as TimeConfig, NaiveProvider as NaiveTime},
    },
};

use crate::Error;

pub mod cursor;
pub mod throttle;

pub(crate) mod bech32;
pub(crate) mod time;

mod facade;

pub(crate) trait SwallowResult {
    fn ok_or_warn(self, context: &'static str);
}

impl SwallowResult for Result<(), Error> {
    fn ok_or_warn(self, context: &'static str) {
        match self {
            Ok(_) => (),
            Err(e) => log::warn!("{}: {:?}", context, e),
        }
    }
}

/// Well-known information about the blockhain network
///
/// Some of the logic in Oura depends on particular characteristic of the
/// network that it's consuming from. For example: time calculation and bech32
/// encoding. This struct groups all of these blockchain network specific
/// values.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub shelley_slot_length: u32,
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
    pub address_hrp: String,
}

impl ChainWellKnownInfo {
    /// Hardcoded values for mainnet
    pub fn mainnet() -> Self {
        ChainWellKnownInfo {
            shelley_slot_length: 1,
            shelley_known_slot: 4492800,
            shelley_known_hash: "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de"
                .to_string(),
            shelley_known_time: 1596059091,
            address_hrp: "addr".to_string(),
        }
    }

    /// Hardcoded values for testnet
    pub fn testnet() -> Self {
        ChainWellKnownInfo {
            shelley_slot_length: 1,
            shelley_known_slot: 1598400,
            shelley_known_hash: "02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f"
                .to_string(),
            shelley_known_time: 1595967616,
            address_hrp: "addr_test".to_string(),
        }
    }

    /// Uses the value of the magic to return either mainnet or testnet
    /// hardcoded values.
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(Self::mainnet()),
            TESTNET_MAGIC => Ok(Self::testnet()),
            _ => Err("can't infer well-known chain infro from specified magic".into()),
        }
    }
}

impl Default for ChainWellKnownInfo {
    fn default() -> Self {
        Self::mainnet()
    }
}

/// Entry point for all shared utilities
pub struct Utils {
    pub(crate) well_known: ChainWellKnownInfo,
    pub(crate) time: Option<NaiveTime>,
    pub(crate) bech32: Bech32Provider,
    pub(crate) cursor: Option<cursor::Provider>,
}

impl Utils {
    // TODO: refactor this using the builder pattern
    pub fn new(well_known: ChainWellKnownInfo, cursor: Option<cursor::Provider>) -> Self {
        Self {
            time: NaiveTime::new(TimeConfig::from_well_known(&well_known)).into(),
            bech32: Bech32Provider::new(Bech32Config::from_well_known(&well_known)),
            cursor,
            well_known,
        }
    }
}

/// Wraps a struct with pipeline-wide utilities
///
/// Most of the stage bootstrapping processes will require a custom config value
/// and a reference to the shared utilities singelton. This is a quality-of-life
/// artifact to wrap other structs (usually configs) and attach the utilities
/// singelton entrypoint.
pub struct WithUtils<C> {
    pub utils: Arc<Utils>,
    pub inner: C,
}

impl<C> WithUtils<C> {
    pub fn new(inner: C, utils: Arc<Utils>) -> Self {
        WithUtils { inner, utils }
    }

    pub fn attach_utils_to<T>(&self, target: T) -> WithUtils<T> {
        WithUtils {
            inner: target,
            utils: self.utils.clone(),
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
