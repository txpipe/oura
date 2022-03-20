use std::ops::Deref;

use log::info;

use pallas::network::{
    miniprotocols::{handshake::n2n, run_agent, MAINNET_MAGIC},
    multiplexer::Channel,
};

use serde::Deserialize;

use crate::{
    mapper::{Config as MapperConfig, EventWriter},
    pipelining::{new_inter_stage_channel, PartialBootstrapResult, SourceProvider},
    sources::{
        common::{AddressArg, MagicArg, PointArg},
        define_start_point, setup_multiplexer, IntersectArg, RetryPolicy,
    },
    utils::{ChainWellKnownInfo, WithUtils},
    Error,
};

use super::run::{fetch_blocks_forever, observe_headers_forever};

#[derive(Debug, Deserialize)]
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
}

fn do_handshake(channel: &mut Channel, magic: u64) -> Result<(), Error> {
    let versions = n2n::VersionTable::v6_and_above(magic);
    let agent = run_agent(n2n::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2n::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version".into()),
    }
}

impl SourceProvider for WithUtils<Config> {
    fn bootstrap(&self) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let mut muxer = setup_multiplexer(
            &self.inner.address.0,
            &self.inner.address.1,
            &[0, 2, 3],
            &self.inner.retry_policy,
        )?;

        let magic = match &self.inner.magic {
            Some(m) => *m.deref(),
            None => MAINNET_MAGIC,
        };

        let writer = EventWriter::new(output_tx, self.utils.clone(), self.inner.mapper.clone());

        let mut hs_channel = muxer.use_channel(0);
        do_handshake(&mut hs_channel, magic)?;

        let mut cs_channel = muxer.use_channel(2);

        let known_points = define_start_point(
            &self.inner.intersect,
            #[allow(deprecated)]
            &self.inner.since,
            &self.utils,
            &mut cs_channel,
        )?;

        info!("starting chain sync from: {:?}", &known_points);

        let (headers_tx, headers_rx) = std::sync::mpsc::sync_channel(100);

        let min_depth = self.inner.min_depth;
        let cs_writer = writer.clone();
        let cs_handle = std::thread::spawn(move || {
            observe_headers_forever(cs_channel, cs_writer, known_points, headers_tx, min_depth)
                .expect("chainsync loop failed");
        });

        let bf_channel = muxer.use_channel(3);
        let bf_writer = writer;
        let _bf_handle = std::thread::spawn(move || {
            fetch_blocks_forever(bf_channel, bf_writer, headers_rx)
                .expect("blockfetch loop failed");
        });

        Ok((cs_handle, output_rx))
    }
}
