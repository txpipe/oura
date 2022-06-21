use serde::Deserialize;

use crate::{
    mapper::Config as MapperConfig,
    pipelining::{new_inter_stage_channel, PartialBootstrapResult, SourceProvider},
    sources::{
        common::{AddressArg, MagicArg, PointArg},
        FinalizeConfig, IntersectArg, RetryPolicy,
    },
    utils::{ChainWellKnownInfo, WithUtils},
};

use super::run::do_chainsync;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,

    #[deprecated(note = "use intersect value instead")]
    pub since: Option<PointArg>,

    pub intersect: Option<IntersectArg>,

    #[deprecated(note = "chain info is now pipeline-wide, use utils")]
    pub well_known: Option<ChainWellKnownInfo>,

    #[serde(default)]
    pub mapper: MapperConfig,

    /// Min block depth (# confirmations) required
    ///
    /// The min depth a block requires to be considered safe to send down the
    /// pipeline. This value is used to configure a rollback buffer used
    /// internally by the stage. A high value (eg: ~6) will reduce the
    /// probability of seeing rollbacks events. The trade-off is that the stage
    /// will need some time to fill up the buffer before sending the 1st event.
    #[serde(default)]
    pub min_depth: usize,

    pub retry_policy: Option<RetryPolicy>,

    pub finalize: Option<FinalizeConfig>,
}

impl SourceProvider for WithUtils<Config> {
    fn bootstrap(&self) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let config = self.inner.clone();
        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            do_chainsync(&config, utils, output_tx)
                .expect("chainsync fails after applying max retry policy")
        });

        Ok((handle, output_rx))
    }
}
