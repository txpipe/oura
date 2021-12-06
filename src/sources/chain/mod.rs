mod map;
mod setup;

pub use setup::*;

use log::error;

use pallas::{
    ledger::alonzo::Block,
    ouroboros::network::{
        chainsync::{BlockBody, ClientConsumer, Observer},
        machines::{primitives::Point, run_agent},
        multiplexer::Channel,
    },
};
use std::{error::Error, sync::mpsc::Sender};

use crate::ports::Event;

use self::map::{EventSource, EventWriter};

#[derive(Debug)]
pub struct ChainObserver(pub Sender<Event>);

impl Observer<BlockBody> for ChainObserver {
    fn on_block(&self, content: &BlockBody) -> Result<(), Box<dyn std::error::Error>> {
        let BlockBody(bytes) = content;
        let block = Block::try_from(&bytes[..]);

        match block {
            Ok(block) => {
                let mut storage = Vec::with_capacity(5 + (block.transaction_bodies.len() * 2));
                let mut writer = EventWriter::new(&mut storage);
                block.write_events(&mut writer);
                let sent = storage
                    .into_iter()
                    .map(|e| self.0.send(e))
                    .collect::<Result<Vec<_>, _>>();

                if let Err(err) = sent {
                    log::error!("{:?}", err)
                }
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(bytes));
            }
        };

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Box<dyn std::error::Error>> {
        println!("rollback to {:#?}", point);
        Ok(())
    }
}

fn observe_forever(
    channel: Channel,
    from: Point,
    output: Sender<Event>,
) -> Result<(), Box<dyn Error>> {
    let observer = ChainObserver(output);
    let agent = ClientConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
