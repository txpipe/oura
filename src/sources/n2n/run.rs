use std::fmt::Debug;

use pallas::{
    ledger::primitives::{probing, Era},
    network::{
        miniprotocols::{blockfetch, chainsync, run_agent, Point},
        multiplexer::Channel,
    },
};

use std::sync::mpsc::{Receiver, SyncSender};

use crate::{mapper::EventWriter, model::EventData, utils::SwallowResult, Error};

use super::headers::MultiEraHeader;

struct Block2EventMapper(EventWriter);

impl Debug for Block2EventMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Block2EventMapper").finish()
    }
}

impl blockfetch::Observer for Block2EventMapper {
    fn on_block_received(&self, body: Vec<u8>) -> Result<(), Error> {
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
                        .crawl_from_shelley_cbor(&body, Some(era))
                        .ok_or_warn("error crawling block for events");
                }
            },
            probing::Outcome::Inconclusive => {
                log::error!("can't infer primitive block from cbor, inconslusive probing. CBOR hex for debubbing: {}", hex::encode(body));
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
        buffer.oldest().map(|x| x.0),
        buffer.latest().map(|x| x.0)
    );
}

impl chainsync::Observer<chainsync::HeaderContent> for &mut ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::HeaderContent,
        _tip: &chainsync::Tip,
    ) -> Result<(), Error> {
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
            self.block_requests.send(point)?;
        }

        log_buffer_state(&self.chain_buffer);

        Ok(())
    }

    fn on_rollback(&mut self, point: &Point) -> Result<(), Error> {
        log::info!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");

                self.event_writer.append(EventData::RollBack {
                    block_slot: point.0,
                    block_hash: hex::encode(&point.1),
                })?;
            }
        }

        log_buffer_state(&self.chain_buffer);

        Ok(())
    }
}

pub(crate) fn fetch_blocks_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    input: Receiver<Point>,
) -> Result<(), Error> {
    let observer = Block2EventMapper(event_writer);
    let agent = blockfetch::OnDemandClient::initial(input, observer);
    let agent = run_agent(agent, &mut channel)?;
    log::warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

pub(crate) fn observe_headers_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    from: Point,
    block_requests: SyncSender<Point>,
    min_depth: usize,
) -> Result<(), Error> {
    let observer = &mut ChainObserver {
        chain_buffer: Default::default(),
        min_depth,
        event_writer,
        block_requests,
    };

    let agent = chainsync::HeaderConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    log::warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
