//! Blockchain time utils
//!
//! Common operations to deal with blockchain time and wallclock conversions

use crate::utils::ChainWellKnownInfo;

/// Abstraction available to stages to deal with blockchain time conversions
pub(crate) trait TimeProvider {
    /// Maps between slots and wallclock
    fn slot_to_wallclock(&self, slot: u64) -> u64;
    fn absolute_slot_to_relative(&self, slot: u64) -> (u64, u64);
}

/// A naive, standalone implementation of a time provider
///
/// This time provider doesn't require any external resources other than an
/// initial config. It works by applying simple slot => wallclock conversion
/// logic from a well-known configured point in the chain, assuming homogeneous
/// slot length from that point forward.
#[derive(Clone)]
pub(crate) struct NaiveProvider {
    config: ChainWellKnownInfo,
    shelley_start_epoch: u64,
}

impl NaiveProvider {
    pub fn new(config: ChainWellKnownInfo) -> Self {
        assert!(
            config.byron_epoch_length > 0,
            "byron epoch length needs to be greater than zero"
        );

        assert!(
            config.shelley_epoch_length > 0,
            "shelley epoch length needs to be greater than zero"
        );

        let (shelley_start_epoch, _) = compute_era_epoch(
            config.shelley_known_slot,
            config.byron_slot_length as u64,
            config.byron_epoch_length as u64,
        );

        NaiveProvider {
            config,
            shelley_start_epoch,
        }
    }
}

#[inline]
fn compute_linear_timestamp(
    known_slot: u64,
    known_time: u64,
    slot_length: u64,
    query_slot: u64,
) -> u64 {
    known_time + (query_slot - known_slot) * slot_length
}

#[inline]
fn compute_era_epoch(era_slot: u64, era_slot_length: u64, era_epoch_length: u64) -> (u64, u64) {
    let epoch = (era_slot * era_slot_length) / era_epoch_length;
    let reminder = era_slot % era_epoch_length;

    (epoch, reminder)
}

impl TimeProvider for NaiveProvider {
    fn slot_to_wallclock(&self, slot: u64) -> u64 {
        let NaiveProvider { config, .. } = self;

        if slot < config.shelley_known_slot {
            compute_linear_timestamp(
                config.byron_known_slot,
                config.byron_known_time,
                config.byron_slot_length as u64,
                slot,
            )
        } else {
            compute_linear_timestamp(
                config.shelley_known_slot,
                config.shelley_known_time,
                config.shelley_slot_length as u64,
                slot,
            )
        }
    }

    fn absolute_slot_to_relative(&self, slot: u64) -> (u64, u64) {
        let NaiveProvider {
            config,
            shelley_start_epoch,
        } = self;

        if slot < config.shelley_known_slot {
            compute_era_epoch(
                slot,
                config.byron_slot_length as u64,
                config.byron_epoch_length as u64,
            )
        } else {
            let era_slot = slot - config.shelley_known_slot;

            let (era_epoch, reminder) = compute_era_epoch(
                era_slot,
                config.shelley_slot_length as u64,
                config.shelley_epoch_length as u64,
            );

            (shelley_start_epoch + era_epoch, reminder)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_slot_matches_timestamp(
        provider: &NaiveProvider,
        slot: u64,
        expected_ts: u64,
        expected_epoch: u64,
        expected_epoch_slot: u64,
    ) {
        let wallclock = provider.slot_to_wallclock(slot);
        assert_eq!(wallclock, expected_ts);

        let (epoch, epoch_slot) = provider.absolute_slot_to_relative(slot);
        assert_eq!(epoch, expected_epoch);
        assert_eq!(epoch_slot, expected_epoch_slot);
    }

    #[test]
    fn naive_provider_matches_mainnet_values() {
        let provider = NaiveProvider::new(ChainWellKnownInfo::mainnet());

        // Byron start, value copied from:
        // https://explorer.cardano.org/en/block?id=f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f
        assert_slot_matches_timestamp(&provider, 0, 1506203091, 0, 0);

        // Byron middle, value copied from:
        // https://explorer.cardano.org/en/block?id=c1b57d58761af4dc3c6bdcb3542170cec6db3c81e551cd68012774d1c38129a3
        assert_slot_matches_timestamp(&provider, 2160007, 1549403231, 100, 7);

        // Shelley start, value copied from:
        // https://explorer.cardano.org/en/block?id=aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de
        assert_slot_matches_timestamp(&provider, 4492800, 1596059091, 208, 0);

        // Shelly middle, value copied from:
        // https://explorer.cardano.org/en/block?id=ca60833847d0e70a1adfa6b7f485766003cf7d96d28d481c20d4390f91b76d68
        assert_slot_matches_timestamp(&provider, 51580240, 1643146531, 316, 431440);

        // Shelly middle, value copied from:
        // https://explorer.cardano.org/en/block?id=ec07c6f74f344062db5340480e5b364aac8bb40768d184c1b1491e05c5bec4c4
        assert_slot_matches_timestamp(&provider, 54605026, 1646171317, 324, 226);
    }

    #[test]
    fn naive_provider_matches_testnet_values() {
        let provider = NaiveProvider::new(ChainWellKnownInfo::testnet());

        // Byron origin, value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=8f8602837f7c6f8b8867dd1cbc1842cf51a27eaed2c70ef48325d00f8efb320f
        assert_slot_matches_timestamp(&provider, 0, 1564010416, 0, 0);

        // Byron start, value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=388a82f053603f3552717d61644a353188f2d5500f4c6354cc1ad27a36a7ea91
        assert_slot_matches_timestamp(&provider, 1031, 1564031036, 0, 1031);

        // Byron middle, value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=66102c0b80e1eebc9cddf9cab43c1bf912e4f1963d6f3b8ff948952f8409e779
        assert_slot_matches_timestamp(&provider, 561595, 1575242316, 25, 129595);

        // Shelley start, value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f
        assert_slot_matches_timestamp(&provider, 1598400, 1595967616, 74, 0);

        // Shelley middle, value copied from:
        // https://explorer.cardano-testnet.iohkdev.io/en/block?id=26a1b5a649309c0c8dd48f3069d9adea5a27edf5171dfb941b708acaf2d76dcd
        assert_slot_matches_timestamp(&provider, 48783593, 1643152809, 183, 97193);
    }
}
