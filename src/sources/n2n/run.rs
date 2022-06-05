use std::fmt::Debug;

use pallas::{
    ledger::primitives::{probing, Era},
    network::{
        miniprotocols::{blockfetch, chainsync, run_agent, Point},
        multiplexer::StdChannel,
    },
};

use std::sync::mpsc::{Receiver, SyncSender};

use crate::{
    mapper::EventWriter,
    sources::{should_finalize, FinalizeConfig},
    utils::SwallowResult,
    Error,
};

use super::headers::MultiEraHeader;

struct Block2EventMapper(EventWriter);

impl Debug for Block2EventMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Block2EventMapper").finish()
    }
}

impl blockfetch::Observer for Block2EventMapper {
    fn on_block_received(&mut self, body: Vec<u8>) -> Result<(), Error> {
        let Self(writer) = self;

        match probing::probe_block_cbor_era(&body) {
            probing::Outcome::Matched(era) => match era {
                Era::Byron => {
                    writer
                        .crawl_from_byron_cbor(&body)
                        .ok_or_warn("error crawling block for events");
                }
                _ => {
                    writer
                        .crawl_from_shelley_cbor(&body, era.into())
                        .ok_or_warn("error crawling block for events");
                }
            },
            // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
            // assumption?
            probing::Outcome::GenesisBlock => {
                writer
                    .crawl_from_byron_cbor(&body)
                    .ok_or_warn("error crawling block for events");
            }
            probing::Outcome::Inconclusive => {
                log::error!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
            }
        }

        Ok(())
    }
}

struct ChainObserver {
    min_depth: usize,
    chain_buffer: chainsync::RollbackBuffer,
    block_requests: SyncSender<Point>,
    event_writer: EventWriter,
    finalize_config: Option<FinalizeConfig>,
    block_count: u64,
}

// workaround to put a stop on excessive debug requirement coming from Pallas
impl Debug for ChainObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainObserver").finish()
    }
}

fn log_buffer_state(buffer: &chainsync::RollbackBuffer) {
    log::info!(
        "rollback buffer state, size: {}, oldest: {:?}, latest: {:?}",
        buffer.size(),
        buffer.oldest(),
        buffer.latest(),
    );
}

impl chainsync::Observer<chainsync::HeaderContent> for &mut ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::HeaderContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Error> {
        // parse the header and extract the point of the chain
        let header = MultiEraHeader::try_from(content)?;
        let point = header.read_cursor()?;

        // track the new point in our memory buffer
        log::info!("rolling forward to point {:?}", point);
        self.chain_buffer.roll_forward(point);

        // see if we have points that already reached certain depth
        let ready = self.chain_buffer.pop_with_depth(self.min_depth);
        log::debug!("found {} points with required min depth", ready.len());

        // request download of blocks for confirmed points
        for point in ready {
            log::debug!("requesting block fetch for point {:?}", point);
            self.block_requests.send(point.clone())?;
            self.block_count += 1;

            // evaluate if we should finalize the thread according to config
            if should_finalize(&self.finalize_config, &point, self.block_count) {
                return Ok(chainsync::Continuation::DropOut);
            }
        }

        log_buffer_state(&self.chain_buffer);

        // notify chain tip to the pipeline metrics
        self.event_writer.utils.track_chain_tip(tip.1);

        Ok(chainsync::Continuation::Proceed)
    }

    fn on_rollback(&mut self, point: &Point) -> Result<chainsync::Continuation, Error> {
        log::info!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");
                self.event_writer.append_rollback_event(point)?;
            }
        }

        log_buffer_state(&self.chain_buffer);

        Ok(chainsync::Continuation::Proceed)
    }
}

pub(crate) fn fetch_blocks_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    input: Receiver<Point>,
) -> Result<(), Error> {
    let observer = Block2EventMapper(event_writer);
    let agent = blockfetch::OnDemandClient::initial(input.iter(), observer);
    let agent = run_agent(agent, &mut channel)?;
    log::debug!("blockfetch agent final state: {:?}", agent.state);

    Ok(())
}

pub(crate) fn observe_headers_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    known_points: Option<Vec<Point>>,
    block_requests: SyncSender<Point>,
    min_depth: usize,
    finalize_config: Option<FinalizeConfig>,
) -> Result<(), Error> {
    let observer = &mut ChainObserver {
        chain_buffer: Default::default(),
        min_depth,
        event_writer,
        block_requests,
        block_count: 0,
        finalize_config,
    };

    let agent = chainsync::HeaderConsumer::initial(known_points, observer);
    let agent = run_agent(agent, &mut channel)?;
    log::debug!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
