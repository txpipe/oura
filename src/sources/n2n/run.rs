use std::fmt::Debug;

use log::{info, warn};

use pallas::{
    ledger::primitives::probing,
    network::{
        miniprotocols::{
            blockfetch,
            chainsync::{self, HeaderContent},
            run_agent, Point,
        },
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

        match probing::probe_block_cbor(&body) {
            probing::BlockInference::Byron => {
                writer
                    .crawl_from_byron_cbor(&body)
                    .ok_or_warn("error crawling block for events");
            }
            probing::BlockInference::Shelley => {
                writer
                    .crawl_from_shelley_cbor(&body)
                    .ok_or_warn("error crawling block for events");
            }
            probing::BlockInference::Inconclusive => {
                log::error!("can't infer primitive block from cbor, inconslusive probing. CBOR hex for debubbing: {}", hex::encode(body));
            }
        }

        Ok(())
    }
}

struct ChainObserver {
    block_requests: SyncSender<Point>,
    event_writer: EventWriter,
}

// workaround to put a stop on excessive debug requirement coming from Pallas
impl Debug for ChainObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainObserver").finish()
    }
}

impl chainsync::Observer<HeaderContent> for ChainObserver {
    fn on_roll_forward(
        &self,
        content: chainsync::HeaderContent,
        _tip: &chainsync::Tip,
    ) -> Result<(), Error> {
        let header = MultiEraHeader::try_from(content)?;
        let cursor = header.read_cursor()?;

        info!("requesting block fetch for point {:?}", cursor);
        self.block_requests.send(cursor)?;

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        info!("rolling block to point {:?}", point);

        self.event_writer.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: hex::encode(&point.1),
        })
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
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

pub(crate) fn observe_headers_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    from: Point,
    block_requests: SyncSender<Point>,
) -> Result<(), Error> {
    let observer = ChainObserver {
        event_writer,
        block_requests,
    };

    let agent = chainsync::HeaderConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
