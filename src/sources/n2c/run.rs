use std::{collections::HashMap, fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use pallas::network::{
    miniprotocols::{chainsync, handshake, run_agent, Point, MAINNET_MAGIC},
    multiplexer::StdChannel,
};

use crate::{
    mapper::EventWriter,
    pipelining::StageSender,
    sources::{
        define_start_point, n2c::blocks::CborHolder, setup_multiplexer, should_finalize,
        FinalizeConfig,
    },
    utils::{retry, Utils},
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

fn observe_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    known_points: Option<Vec<Point>>,
    min_depth: usize,
    finalize_config: Option<FinalizeConfig>,
) -> Result<(), AttemptError> {
    let observer = ChainObserver {
        chain_buffer: Default::default(),
        blocks: HashMap::new(),
        min_depth,
        event_writer,
        block_count: 0,
        finalize_config,
    };

    let agent = chainsync::BlockConsumer::initial(known_points, observer);

    match run_agent(agent, &mut channel) {
        Ok(agent) => {
            log::debug!("chainsync agent final state: {:?}", agent.state);
            Ok(())
        }
        Err(err) => Err(AttemptError::Recoverable(err.into())),
    }
}

#[derive(Debug)]
enum AttemptError {
    Recoverable(Error),
    Other(Error),
}

fn do_handshake(channel: &mut StdChannel, magic: u64) -> Result<(), AttemptError> {
    let versions = handshake::n2c::VersionTable::v1_and_above(magic);

    match run_agent(handshake::Initiator::initial(versions), channel) {
        Ok(agent) => match agent.output {
            handshake::Output::Accepted(_, _) => Ok(()),
            _ => Err(AttemptError::Other(
                "couldn't agree on handshake version".into(),
            )),
        },
        Err(err) => Err(AttemptError::Recoverable(err.into())),
    }
}

fn do_chainsync_attempt(
    config: &super::Config,
    utils: Arc<Utils>,
    output_tx: &StageSender,
) -> Result<(), AttemptError> {
    let magic = match config.magic.as_ref() {
        Some(m) => *m.deref(),
        None => MAINNET_MAGIC,
    };

    let mut plexer = setup_multiplexer(&config.address.0, &config.address.1, &config.retry_policy)
        .map_err(|x| AttemptError::Recoverable(x))?;

    let mut hs_channel = plexer.use_channel(0);
    let mut cs_channel = plexer.use_channel(5);

    plexer.muxer.spawn();
    plexer.demuxer.spawn();

    do_handshake(&mut hs_channel, magic)?;

    let known_points = define_start_point(
        &config.intersect,
        #[allow(deprecated)]
        &config.since,
        &utils,
        &mut cs_channel,
    )
    .map_err(|err| AttemptError::Recoverable(err))?;

    log::info!("starting chain sync from: {:?}", &known_points);

    let writer = EventWriter::new(output_tx.clone(), utils, config.mapper.clone());

    observe_forever(
        cs_channel,
        writer,
        known_points,
        config.min_depth,
        config.finalize.clone(),
    )?;

    Ok(())
}

pub fn do_chainsync(
    config: &super::Config,
    utils: Arc<Utils>,
    output_tx: StageSender,
) -> Result<(), Error> {
    retry::retry_operation(
        || match do_chainsync_attempt(config, utils.clone(), &output_tx) {
            Ok(()) => Ok(()),
            Err(AttemptError::Other(msg)) => {
                log::error!("N2N error: {}", msg);
                log::warn!("unrecoverable error performing chainsync, will exit");
                Ok(())
            }
            Err(AttemptError::Recoverable(err)) => Err(err),
        },
        &retry::Policy {
            max_retries: config
                .retry_policy
                .as_ref()
                .map(|x| x.chainsync_max_retries)
                .unwrap_or(50),
            backoff_unit: Duration::from_secs(1),
            backoff_factor: 2,
            max_backoff: config
                .retry_policy
                .as_ref()
                .map(|x| x.chainsync_max_backoff as u64)
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(60)),
        },
    )
}
