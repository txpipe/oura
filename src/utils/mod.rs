//! Pipeline-wide utilities
//!
//! This module includes general-purpose utilities that could potentially be
//! used by more than a single stage. The entry point to this utilities is
//! designed as singleton [`Utils`] instance shared by all stages through an Arc
//! pointer.

use std::sync::Arc;

use pallas::network::miniprotocols::{Point, MAINNET_MAGIC, TESTNET_MAGIC};

use serde::{Deserialize, Serialize};

use crate::{
    model::Event,
    utils::{
        bech32::{Bech32Config, Bech32Provider},
        time::NaiveProvider as NaiveTime,
    },
};

use crate::Error;

pub mod cursor;
pub mod metrics;
pub mod throttle;

pub(crate) mod bech32;
pub(crate) mod retry;
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
    pub byron_epoch_length: u32,
    pub byron_slot_length: u32,
    pub byron_known_slot: u64,
    pub byron_known_hash: String,
    pub byron_known_time: u64,
    pub shelley_epoch_length: u32,
    pub shelley_slot_length: u32,
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
    pub address_hrp: String,
    pub adahandle_policy: String,
}

impl ChainWellKnownInfo {
    /// Hardcoded values for mainnet
    pub fn mainnet() -> Self {
        ChainWellKnownInfo {
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1506203091,
            byron_known_hash: "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 4492800,
            shelley_known_hash: "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de"
                .to_string(),
            shelley_known_time: 1596059091,
            address_hrp: "addr".to_string(),
            adahandle_policy: "f0ff48bbb7bbe9d59a40f1ce90e9e9d0ff5002ec48f232b49ca0fb9a"
                .to_string(),
        }
    }

    /// Hardcoded values for testnet
    pub fn testnet() -> Self {
        ChainWellKnownInfo {
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1564010416,
            byron_known_hash: "8f8602837f7c6f8b8867dd1cbc1842cf51a27eaed2c70ef48325d00f8efb320f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 1598400,
            shelley_known_hash: "02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f"
                .to_string(),
            shelley_known_time: 1595967616,
            address_hrp: "addr_test".to_string(),
            adahandle_policy: "8d18d786e92776c824607fd8e193ec535c79dc61ea2405ddf3b09fe3"
                .to_string(),
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
    pub(crate) metrics: Option<metrics::Provider>,
}

// TODO: refactor this using the builder pattern
impl Utils {
    pub fn new(well_known: ChainWellKnownInfo) -> Self {
        Self {
            time: NaiveTime::new(well_known.clone()).into(),
            bech32: Bech32Provider::new(Bech32Config::from_well_known(&well_known)),
            well_known,
            cursor: None,
            metrics: None,
        }
    }

    pub fn with_cursor(self, config: cursor::Config) -> Self {
        let provider = cursor::Provider::initialize(config);

        Self {
            cursor: provider.into(),
            ..self
        }
    }

    pub fn with_metrics(self, config: metrics::Config) -> Self {
        let provider = metrics::Provider::initialize(&config).expect("metric server started");

        Self {
            metrics: provider.into(),
            ..self
        }
    }
}

/// Wraps a struct with pipeline-wide utilities
///
/// Most of the stage bootstrapping processes will require a custom config value
/// and a reference to the shared utilities singleton. This is a quality-of-life
/// artifact to wrap other structs (usually configs) and attach the utilities
/// singleton entrypoint.
#[derive(Clone)]
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
        let out = Point::Specific(
            other.shelley_known_slot,
            hex::decode(other.shelley_known_hash)?,
        );

        Ok(out)
    }
}
