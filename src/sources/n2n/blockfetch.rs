use pallas::crypto::hash::Hash;
use pallas::network::miniprotocols::blockfetch;
use pallas::network::miniprotocols::Point;
use tracing::{debug, error, instrument};

use crate::prelude::*;

pub type UpstreamPort = gasket::messaging::TwoPhaseInputPort<ChainSyncEvent>;
pub type DownstreamPort = gasket::messaging::OutputPort<BlockFetchEvent>;

pub type OuroborosClient = blockfetch::Client<ProtocolChannel>;

pub struct Worker {
    client: OuroborosClient,
    upstream: UpstreamPort,
    downstream: DownstreamPort,
    block_count: gasket::metrics::Counter,
}

impl Worker {
    pub fn new(
        plexer: ProtocolChannel,
        upstream: UpstreamPort,
        downstream: DownstreamPort,
    ) -> Self {
        let client = OuroborosClient::new(plexer);

        Self {
            client,
            upstream,
            downstream,
            block_count: Default::default(),
        }
    }

    #[instrument(skip(self))]
    fn fetch_block(&mut self, slot: u64, hash: Hash<32>) -> Result<Vec<u8>, gasket::error::Error> {
        match self
            .client
            .fetch_single(Point::Specific(slot, hash.to_vec()))
        {
            Ok(x) => {
                debug!("block fetch succeded");
                Ok(x)
            }
            Err(blockfetch::Error::ChannelError(x)) => {
                error!("plexer channel error: {}", x);
                Err(gasket::error::Error::RetryableError)
            }
            Err(x) => {
                error!("unrecoverable block fetch error: {}", x);
                Err(gasket::error::Error::WorkPanic)
            }
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("fetched_blocks", &self.block_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.upstream.recv_or_idle()?;

        let msg = match msg.payload {
            ChainSyncEvent::RollForward(s, h) => {
                let body = self.fetch_block(s, h)?;
                BlockFetchEvent::RollForward(s, h, body)
            }
            ChainSyncEvent::Rollback(x) => BlockFetchEvent::Rollback(x),
        };

        self.downstream.send(msg.into())?;

        // remove the processed event from the queue
        self.upstream.commit();

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
