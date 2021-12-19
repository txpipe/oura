mod setup;

pub use setup::*;

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

use std::sync::mpsc::Sender;

use crate::{
    framework::{ChainWellKnownInfo, Error, Event, EventData, EventSource, EventWriter},
    mapping::ToHex,
};

#[derive(Debug)]
pub struct Content(Block);

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
        let hash = crypto::hash_block_header(&self.0.header)?;
        Ok(Point(self.0.header.header_body.slot, Vec::from(hash)))
    }
}

#[derive(Debug)]
pub struct ChainObserver(EventWriter);

impl Observer<Content> for ChainObserver {
    fn on_block(
        &self,
        _cursor: &Option<Point>,
        content: &Content,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Content(block) = content;
        block.write_events(&self.0)?;

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        self.0.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: point.1.to_hex(),
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

fn observe_forever(
    mut channel: Channel,
    chain_info: ChainWellKnownInfo,
    from: Point,
    output: Sender<Event>,
) -> Result<(), Error> {
    let writer = EventWriter::new(output, Some(chain_info));
    let observer = ChainObserver(writer);
    let agent = Consumer::<Content, _>::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
