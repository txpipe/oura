use std::{collections::HashMap, fmt::Debug};

use pallas::network::{
    miniprotocols::{chainsync, run_agent, Point},
    multiplexer::StdChannel,
};

use crate::{
    mapper::EventWriter,
    sources::{n2c::blocks::CborHolder, should_finalize, FinalizeConfig},
    Error,
};

use super::blocks::MultiEraBlock;

struct ChainObserver {
    chain_buffer: chainsync::RollbackBuffer,
    min_depth: usize,
    blocks: HashMap<Point, CborHolder>,
    event_writer: EventWriter,
    finalize_config: Option<FinalizeConfig>,
    block_count: u64,
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
        buffer.oldest(),
        buffer.latest(),
    );
}

impl<'b> chainsync::Observer<chainsync::BlockContent> for ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::BlockContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Box<dyn std::error::Error>> {
        // parse the block and extract the point of the chain
        let cbor = content.into();
        let block = CborHolder::new(cbor);
        let point = block.parse()?.read_cursor()?;

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

            match block.parse()? {
                MultiEraBlock::Byron(model) => self
                    .event_writer
                    .crawl_byron_with_cbor(&model, block.cbor())?,
                MultiEraBlock::AlonzoCompatible(model, era) => self
                    .event_writer
                    .crawl_shelley_with_cbor(&model, block.cbor(), era.into())?,
            };

            self.block_count += 1;

            // evaluate if we should finalize the thread according to config
            if should_finalize(&self.finalize_config, &point, self.block_count) {
                return Ok(chainsync::Continuation::DropOut);
            }
        }

        log_buffer_state(&self.chain_buffer);

        // notify chain tip to the pipeline metrics
        self.event_writer.utils.track_chain_tip(tip.1);

        Ok(chainsync::Continuation::Proceed)
    }

    fn on_rollback(&mut self, point: &Point) -> Result<chainsync::Continuation, Error> {
        log::info!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);

                // drain memory blocks afther the rollback slot
                self.blocks
                    .retain(|x, _| x.slot_or_default() <= point.slot_or_default());
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");

                // clear all the blocks in memory, they are orphan
                self.blocks.clear();

                self.event_writer.append_rollback_event(point)?;
            }
        }

        log_buffer_state(&self.chain_buffer);

        Ok(chainsync::Continuation::Proceed)
    }
}

pub(crate) fn observe_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    known_points: Option<Vec<Point>>,
    min_depth: usize,
    finalize_config: Option<FinalizeConfig>,
) -> Result<(), Error> {
    let observer = ChainObserver {
        chain_buffer: Default::default(),
        blocks: HashMap::new(),
        min_depth,
        event_writer,
        block_count: 0,
        finalize_config,
    };

    let agent = chainsync::BlockConsumer::initial(known_points, observer);
    let agent = run_agent(agent, &mut channel)?;
    log::warn!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}
