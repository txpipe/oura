use minicbor::data::Tag;
use std::fmt::Debug;

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

use std::sync::mpsc::{Receiver, SyncSender};

use crate::{mapper::EventWriter, model::EventData, utils::SwallowResult, Error};

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
        let hash = crypto::hash_block_header(&self.1);
        Ok(Point(self.1.header_body.slot, hash.to_vec()))
    }
}

struct Block2EventMapper(EventWriter);

impl Debug for Block2EventMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Block2EventMapper").finish()
    }
}

impl BlockObserver for Block2EventMapper {
    fn on_block_received(&self, body: Vec<u8>) -> Result<(), Error> {
        let maybe_block = alonzo::BlockWrapper::decode_fragment(&body[..]);

        match maybe_block {
            Ok(alonzo::BlockWrapper(_, block)) => {
                let Self(writer) = self;

                writer
                    .crawl(&block)
                    .ok_or_warn("error crawling block for events");
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(body));
            }
        };

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
    let agent = BlockClient::initial(input, observer);
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

    let agent = Consumer::<Content, _>::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
