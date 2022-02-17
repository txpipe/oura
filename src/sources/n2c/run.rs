use std::{collections::HashMap, fmt::Debug, ops::Deref};

use pallas::network::{
    miniprotocols::{chainsync, run_agent, Point},
    multiplexer::Channel,
};

use crate::{mapper::EventWriter, model::EventData, Error};

use super::blocks::MultiEraBlock;

struct ChainObserver {
    chain_buffer: chainsync::RollbackBuffer,
    min_depth: usize,
    blocks: HashMap<Point, MultiEraBlock>,
    event_writer: EventWriter,
}

// workaround to put a stop on excessive debug requirement coming from Pallas
impl Debug for ChainObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChainObserver").finish()
    }
}

fn log_buffer_state(buffer: &chainsync::RollbackBuffer) {
    log::info!(
        "rollback buffer state, size: {}, oldest: {:?}, latest: {:?}",
        buffer.size(),
        buffer.oldest().map(|x| x.0),
        buffer.latest().map(|x| x.0)
    );
}

impl chainsync::Observer<chainsync::BlockContent> for ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::BlockContent,
        _tip: &chainsync::Tip,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // parse the block and extract the point of the chain
        let cbor = Vec::from(content.deref());
        let block = MultiEraBlock::try_from(content)?;
        let point = block.read_cursor()?;

        // store the block for later retrieval
        self.blocks.insert(point.clone(), block);

        // track the new point in our memory buffer
        log::info!("rolling forward to point {:?}", point);
        self.chain_buffer.roll_forward(point);

        // see if we have points that already reached certain depth
        let ready = self.chain_buffer.pop_with_depth(self.min_depth);
        log::debug!("found {} points with required min depth", ready.len());

        // find confirmed block in memory and send down the pipeline
        for point in ready {
            let block = self
                .blocks
                .remove(&point)
                .expect("required block not found in memory");

            match block {
                MultiEraBlock::Byron(model) => {
                    self.event_writer.crawl_byron_with_cbor(&model, &cbor)?
                }
                MultiEraBlock::Shelley(model) => {
                    self.event_writer.crawl_shelley_with_cbor(&model, &cbor)?
                }
            };
        }

        log_buffer_state(&self.chain_buffer);

        Ok(())
    }

    fn on_rollback(&mut self, point: &Point) -> Result<(), Error> {
        log::info!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);

                // drain memory blocks afther the rollback slot
                self.blocks.retain(|x, _| x.0 <= point.0);
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");

                // clear all the blocks in memory, they are orphan
                self.blocks.clear();

                self.event_writer.append(EventData::RollBack {
                    block_slot: point.0,
                    block_hash: hex::encode(&point.1),
                })?;
            }
        }

        log_buffer_state(&self.chain_buffer);

        Ok(())
    }
}

pub(crate) fn observe_forever(
    mut channel: Channel,
    event_writer: EventWriter,
    from: Vec<Point>,
    min_depth: usize,
) -> Result<(), Error> {
    let observer = ChainObserver {
        chain_buffer: Default::default(),
        blocks: HashMap::new(),
        min_depth,
        event_writer,
    };

    let agent = chainsync::BlockConsumer::initial(from, observer);
    let agent = run_agent(agent, &mut channel)?;
    log::warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
