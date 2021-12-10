mod setup;

pub use setup::*;

use log::{error, info};

use pallas::{
    ledger::alonzo::{BlockWrapper, Fragment},
    ouroboros::network::{
        blockfetch::{Observer as BlockObserver, OnDemandClient as BlockClient},
        chainsync::{NodeConsumer, Observer, WrappedHeader},
        machines::{primitives::Point, run_agent},
        multiplexer::Channel,
    },
};

use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender},
};

use crate::framework::{Event, EventSource, EventWriter};

#[derive(Debug)]
pub struct ChainObserver(pub Sender<Point>);

impl Observer<WrappedHeader> for ChainObserver {
    fn on_block(
        &self,
        cursor: &Option<Point>,
        _content: &WrappedHeader,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("requesting block fetch for point {:?}", cursor);

        if let Some(cursor) = cursor {
            self.0.send(cursor.clone())?;
        }

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Box<dyn std::error::Error>> {
        println!("rollback to {:#?}", point);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Block2EventMapper(Sender<Event>);

impl BlockObserver for Block2EventMapper {
    fn on_block(&self, body: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let maybe_block = BlockWrapper::decode_fragment(&body[..]);

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
                log::info!("{}", hex::encode(body));
            }
        };

        Ok(())
    }
}

fn fetch_blocks_forever(
    mut channel: Channel,
    input: Receiver<Point>,
    output: Sender<Event>,
) -> Result<(), Box<dyn Error>> {
    let observer = Block2EventMapper(output);
    let agent = BlockClient::initial(input, observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

fn observe_headers_forever(
    mut channel: Channel,
    from: Point,
    output: Sender<Point>,
) -> Result<(), Box<dyn Error>> {
    let observer = ChainObserver(output);
    let agent = NodeConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
