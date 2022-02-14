use std::{fmt::Debug, ops::Deref};

use log::{error, info};

use pallas::network::{
    miniprotocols::{chainsync, run_agent, Point},
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

impl Deref for ChainObserver {
    type Target = EventWriter;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ChainObserver {
    fn new(writer: EventWriter) -> Self {
        Self(writer)
    }
}

impl chainsync::Observer<chainsync::BlockContent> for ChainObserver {
    fn on_roll_forward(
        &self,
        content: chainsync::BlockContent,
        _tip: &chainsync::Tip,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cbor = Vec::from(content.deref());
        let block = MultiEraBlock::try_from(content)?;

        match block {
            MultiEraBlock::Byron(model) => self.crawl_byron_with_cbor(&model, &cbor)?,
            MultiEraBlock::Shelley(model) => self.crawl_shelley_with_cbor(&model, &cbor)?,
        };

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Error> {
        self.0.append(EventData::RollBack {
            block_slot: point.0,
            block_hash: hex::encode(&point.1),
        })
    }

    fn on_intersect_found(&self, point: &Point, _tip: &chainsync::Tip) -> Result<(), Error> {
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
    let agent = chainsync::BlockConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, &mut channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
