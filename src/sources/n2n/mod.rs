mod setup;

pub use setup::*;

use log::{info, warn};

use pallas::{
    ledger::alonzo::{BlockWrapper, Fragment},
    ouroboros::network::{
        blockfetch::{Observer as BlockObserver, OnDemandClient as BlockClient},
        chainsync::{NodeConsumer, Observer, WrappedHeader},
        machines::{primitives::Point, run_agent},
        multiplexer::Channel,
    },
};

use std::sync::mpsc::{Receiver, Sender};

use crate::{
    framework::{Error, Event, EventData, EventSource, EventWriter},
    mapping::ToHex,
};

#[derive(Debug)]
pub struct Block2EventMapper(EventWriter);

impl BlockObserver for Block2EventMapper {
    fn on_block_received(&self, body: Vec<u8>) -> Result<(), Error> {
        let maybe_block = BlockWrapper::decode_fragment(&body[..]);

        match maybe_block {
            Ok(BlockWrapper(_, block)) => {
                block.write_events(&self.0)?;
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

impl Observer<WrappedHeader> for ChainObserver {
    fn on_block(&self, cursor: &Option<Point>, _content: &WrappedHeader) -> Result<(), Error> {
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
    input: Receiver<Point>,
    output: Sender<Event>,
) -> Result<(), Error> {
    let writer = EventWriter::new(output);
    let observer = Block2EventMapper(writer);
    let agent = BlockClient::initial(input, observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

fn observe_headers_forever(
    mut channel: Channel,
    from: Point,
    event_output: Sender<Event>,
    block_requests: Sender<Point>,
) -> Result<(), Error> {
    let event_writer = EventWriter::new(event_output);
    let observer = ChainObserver {
        event_writer,
        block_requests,
    };
    let agent = NodeConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
