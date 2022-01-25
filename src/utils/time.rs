//! Blockchain time utils
//!
//! Common operations to deal with blockchain time and wallclock conversions

use serde::Deserialize;

use crate::Error;

/// Abstraction available to stages to deal with blockchain time conversions
pub(crate) trait TimeProvider {
    /// Maps between slots and wallclock
    fn slot_to_wallclock(&self, slot: u64) -> Result<u64, Error>;
}

#[derive(Deserialize, Clone)]
pub struct NaiveConfig {
    pub slot_length: u32,
    pub start_slot: u64,
    pub start_timestamp: u64,
}

/// A naive, standalone implementation of a time provider
///
/// This time provider doesn't require any external resources other than an
/// initial config. It works by applying simple slot => wallclock conversion
/// logic from a well-known configured point in the chain, assuming homogeneous
/// slot length from that point forward.
#[derive(Clone)]
pub(crate) struct NaiveProvider(NaiveConfig);

impl NaiveProvider {
    pub fn new(config: NaiveConfig) -> Self {
        NaiveProvider(config)
    }
}

impl TimeProvider for NaiveProvider {
    fn slot_to_wallclock(&self, slot: u64) -> Result<u64, Error> {
        let NaiveProvider(config) = self;

        if slot < config.start_slot {
            return Err(
                "naive time provider can't compute wallclock for slots after start_slot".into(),
            );
        }

        let total_delta_secs = (slot - config.start_slot) * config.slot_length as u64;

        let out = config.start_timestamp + total_delta_secs;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shelley_mainnet() -> NaiveConfig {
        NaiveConfig {
            slot_length: 1,
            start_slot: 4492800,
            start_timestamp: 1596059091,
        }
    }

    fn shelley_testnet() -> NaiveConfig {
        NaiveConfig {
            slot_length: 1,
            start_slot: 1598400,
            start_timestamp: 1595967616,
        }
    }

    fn assert_slot_matches_timestamp(provider: &NaiveProvider, slot: u64, ts: u64) {
        let wallclock = provider
            .slot_to_wallclock(slot)
            .expect("unable to compute wallclock");

        assert_eq!(wallclock, ts);
    }

    #[test]
    fn naive_provider_matches_mainnet_values() {
        let provider = NaiveProvider::new(shelley_mainnet());

        // value copied from:
        // https://explorer.cardano.org/en/block?id=aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de
        assert_slot_matches_timestamp(&provider, 4492800, 1596059091);

        // value copied from:
        // https://explorer.cardano.org/en/block?id=ca60833847d0e70a1adfa6b7f485766003cf7d96d28d481c20d4390f91b76d68
        assert_slot_matches_timestamp(&provider, 51580240, 1643146531);
    }

    #[test]
    fn naive_provider_matches_testnet_values() {
        let provider = NaiveProvider::new(shelley_testnet());

        // value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f
        assert_slot_matches_timestamp(&provider, 1598400, 1595967616);

        // value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=26a1b5a649309c0c8dd48f3069d9adea5a27edf5171dfb941b708acaf2d76dcd
        assert_slot_matches_timestamp(&provider, 48783593, 1643152809);
    }
}
