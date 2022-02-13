use std::fmt::Debug;

use log::{error, info};

use pallas::network::{
    miniprotocols::{
        chainsync::{Consumer, Observer, Tip},
        run_agent, Point,
    },
    multiplexer::Channel,
};

use crate::{mapper::EventWriter, model::EventData, Error};

use super::blocks::MultiEraBlock;

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

impl Observer<MultiEraBlock> for ChainObserver {
    fn on_block(
        &self,
        _cursor: &Option<Point>,
        content: &MultiEraBlock,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Self(writer) = self;

        match content {
            MultiEraBlock::Byron(model, cbor) => writer.crawl_byron_with_cbor(model, cbor)?,
            MultiEraBlock::Shelley(model, cbor) => writer.crawl_shelley_with_cbor(model, cbor)?,
        };

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
    let agent = Consumer::<MultiEraBlock, _>::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
