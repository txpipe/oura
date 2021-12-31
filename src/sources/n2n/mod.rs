mod setup;

use minicbor::data::Tag;
pub use setup::*;

use log::{info, warn};

use pallas::{
    ledger::alonzo::{self, crypto, Fragment, Header},
    ouroboros::network::{
        blockfetch::{Observer as BlockObserver, OnDemandClient as BlockClient},
        chainsync::{BlockLike, Consumer, Observer},
        machines::{
            primitives::Point, run_agent, DecodePayload, EncodePayload, PayloadDecoder,
            PayloadEncoder,
        },
        multiplexer::Channel,
    },
};

use std::sync::mpsc::{Receiver, Sender};

use crate::{
    framework::{Error, EventContext, EventData, EventSource, EventWriter},
    mapping::ToHex,
};

#[derive(Debug)]
pub struct Content(u32, Header);

impl EncodePayload for Content {
    fn encode_payload(&self, e: &mut PayloadEncoder) -> Result<(), Box<dyn std::error::Error>> {
        e.array(2)?;
        e.u32(self.0)?;
        e.tag(Tag::Cbor)?;
        e.bytes(&self.1.encode_fragment()?)?;

        Ok(())
    }
}

impl DecodePayload for Content {
    fn decode_payload(d: &mut PayloadDecoder) -> Result<Self, Box<dyn std::error::Error>> {
        d.array()?;
        let unknown = d.u32()?; // WTF is this value?
        d.tag()?;
        let bytes = d.bytes()?;
        let header = Header::decode_fragment(bytes)?;
        Ok(Content(unknown, header))
    }
}

impl BlockLike for Content {
    fn block_point(&self) -> Result<Point, Box<dyn std::error::Error>> {
        let hash = crypto::hash_block_header(&self.1)?;
        Ok(Point(self.1.header_body.slot, Vec::from(hash)))
    }
}

#[derive(Debug)]
pub struct Block2EventMapper(EventWriter);

impl BlockObserver for Block2EventMapper {
    fn on_block_received(&self, body: Vec<u8>) -> Result<(), Error> {
        let maybe_block = alonzo::BlockWrapper::decode_fragment(&body[..]);

        match maybe_block {
            Ok(alonzo::BlockWrapper(_, block)) => {
                // inject the block hash into the context for nested events
                let hash = crypto::hash_block_header(&block.header)?;

                let writer = self.0.child_writer(EventContext {
                    block_hash: Some(hex::encode(hash)),
                    ..EventContext::default()
                });

                block.write_events(&writer)?;
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(body));
            }
        };

        Ok(())
    }
}

#[derive(Debug)]
pub struct ChainObserver {
    block_requests: Sender<Point>,
    event_writer: EventWriter,
}

impl Observer<Content> for ChainObserver {
    fn on_block(&self, cursor: &Option<Point>, _content: &Content) -> Result<(), Error> {
        info!("requesting block fetch for point {:?}", cursor);

        if let Some(cursor) = cursor {
            self.block_requests.send(cursor.clone())?;
        }

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        info!("rolling block to point {:?}", point);

        self.event_writer.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: point.1.to_hex(),
        })
    }
}

fn fetch_blocks_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    input: Receiver<Point>,
) -> Result<(), Error> {
    let observer = Block2EventMapper(event_writer);
    let agent = BlockClient::initial(input, observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

fn observe_headers_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    from: Point,
    block_requests: Sender<Point>,
) -> Result<(), Error> {
    let observer = ChainObserver {
        event_writer,
        block_requests,
    };

    let agent = Consumer::<Content, _>::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
