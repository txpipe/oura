mod setup;

pub use setup::*;

use log::error;

use pallas::{
    ledger::alonzo::{BlockWrapper, Fragment},
    ouroboros::network::{
        chainsync::{BlockBody, ClientConsumer, Observer, Tip},
        machines::{primitives::Point, run_agent},
        multiplexer::Channel,
    },
};

use std::sync::mpsc::Sender;

use crate::{
    framework::{Error, Event, EventData, EventSource, EventWriter},
    mapping::ToHex,
};

#[derive(Debug)]
pub struct ChainObserver(EventWriter);

impl Observer<BlockBody> for ChainObserver {
    fn on_block(
        &self,
        _cursor: &Option<Point>,
        content: &BlockBody,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let BlockBody(bytes) = content;
        let maybe_block = BlockWrapper::decode_fragment(&bytes[..]);

        match maybe_block {
            Ok(BlockWrapper(_, block)) => {
                block.write_events(&self.0)?;
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(bytes));
            }
        };

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        self.0.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: point.1.to_hex(),
        })
    }

    fn on_intersect_found(&self, point: &Point, _tip: &Tip) -> Result<(), Error> {
        println!("intersect found {:#?}", point);
        Ok(())
    }

    fn on_tip_reached(&self) -> Result<(), Error> {
        println!("tip reached");
        Ok(())
    }
}

fn observe_forever(mut channel: Channel, from: Point, output: Sender<Event>) -> Result<(), Error> {
    let writer = EventWriter::new(output);
    let observer = ChainObserver(writer);
    let agent = ClientConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
