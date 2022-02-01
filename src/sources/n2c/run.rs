use std::fmt::Debug;

use log::{error, info};

use pallas::{
    ledger::alonzo::{crypto, Block, BlockWrapper, Fragment},
    ouroboros::network::{
        chainsync::{BlockLike, Consumer, Observer, Tip},
        machines::{
            primitives::Point, run_agent, DecodePayload, EncodePayload, PayloadDecoder,
            PayloadEncoder,
        },
        multiplexer::Channel,
    },
};

use crate::{mapper::EventWriter, model::EventData, utils::SwallowResult, Error};

#[derive(Debug)]
struct Content(Block);

impl EncodePayload for Content {
    fn encode_payload(&self, _e: &mut PayloadEncoder) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

impl DecodePayload for Content {
    fn decode_payload(d: &mut PayloadDecoder) -> Result<Self, Box<dyn std::error::Error>> {
        d.tag()?;
        let bytes = d.bytes()?;
        let BlockWrapper(_, block) = BlockWrapper::decode_fragment(bytes)?;
        Ok(Content(block))
    }
}

impl BlockLike for Content {
    fn block_point(&self) -> Result<Point, Box<dyn std::error::Error>> {
        let hash = crypto::hash_block_header(&self.0.header);
        Ok(Point(self.0.header.header_body.slot, hash.to_vec()))
    }
}

struct ChainObserver(EventWriter);

// workaround to put a stop on excessive debug requirement coming from Pallas
impl Debug for ChainObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainObserver").finish()
    }
}

impl ChainObserver {
    fn new(writer: EventWriter) -> Self {
        Self(writer)
    }
}

impl Observer<Content> for ChainObserver {
    fn on_block(
        &self,
        _cursor: &Option<Point>,
        content: &Content,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Self(writer) = self;
        let Content(block) = content;

        writer
            .crawl(block)
            .ok_or_warn("error crawling block for events");

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        self.0.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: hex::encode(&point.1),
        })
    }

    fn on_intersect_found(&self, point: &Point, _tip: &Tip) -> Result<(), Error> {
        info!("intersect found {:?}", point);
        Ok(())
    }

    fn on_tip_reached(&self) -> Result<(), Error> {
        info!("tip reached");
        Ok(())
    }
}

pub(crate) fn observe_forever(
    mut channel: Channel,
    writer: EventWriter,
    from: Point,
) -> Result<(), Error> {
    let observer = ChainObserver::new(writer);
    let agent = Consumer::<Content, _>::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
