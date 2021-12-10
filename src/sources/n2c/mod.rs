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

use std::{error::Error, sync::mpsc::Sender};

use crate::framework::{Event, EventSource, EventWriter};

#[derive(Debug)]
pub struct ChainObserver(pub Sender<Event>);

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

    fn on_intersect_found(
        &self,
        point: &Point,
        _tip: &Tip,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("rollback to {:#?}", point);
        Ok(())
    }

    fn on_tip_reached(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("tip reached");
        Ok(())
    }
}

fn observe_forever(
    mut channel: Channel,
    from: Point,
    output: Sender<Event>,
) -> Result<(), Box<dyn Error>> {
    let observer = ChainObserver(output);
    let agent = ClientConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
