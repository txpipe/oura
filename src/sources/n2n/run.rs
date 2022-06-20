use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use pallas::{
    ledger::primitives::{probing, Era},
    network::{
        miniprotocols::{blockfetch, chainsync, handshake, run_agent, Point, MAINNET_MAGIC},
        multiplexer::StdChannel,
    },
};

use std::sync::mpsc::{Receiver, SyncSender};

use crate::{
    mapper::EventWriter,
    pipelining::StageSender,
    sources::{define_start_point, setup_multiplexer, should_finalize, FinalizeConfig},
    utils::{retry, SwallowResult, Utils},
    Error,
};

use super::headers::MultiEraHeader;

struct Block2EventMapper(EventWriter);

impl Debug for Block2EventMapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Block2EventMapper").finish()
    }
}

impl blockfetch::Observer for Block2EventMapper {
    fn on_block_received(&mut self, body: Vec<u8>) -> Result<(), Error> {
        let Self(writer) = self;

        match probing::probe_block_cbor_era(&body) {
            probing::Outcome::Matched(era) => match era {
                Era::Byron => {
                    writer
                        .crawl_from_byron_cbor(&body)
                        .ok_or_warn("error crawling block for events");
                }
                _ => {
                    writer
                        .crawl_from_shelley_cbor(&body, era.into())
                        .ok_or_warn("error crawling block for events");
                }
            },
            // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
            // assumption?
            probing::Outcome::GenesisBlock => {
                writer
                    .crawl_from_byron_cbor(&body)
                    .ok_or_warn("error crawling block for events");
            }
            probing::Outcome::Inconclusive => {
                log::error!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
            }
        }

        Ok(())
    }
}

struct ChainObserver {
    min_depth: usize,
    chain_buffer: chainsync::RollbackBuffer,
    block_requests: SyncSender<Point>,
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

impl chainsync::Observer<chainsync::HeaderContent> for &mut ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::HeaderContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Error> {
        // parse the header and extract the point of the chain
        let header = MultiEraHeader::try_from(content)?;
        let point = header.read_cursor()?;

        // track the new point in our memory buffer
        log::info!("rolling forward to point {:?}", point);
        self.chain_buffer.roll_forward(point);

        // see if we have points that already reached certain depth
        let ready = self.chain_buffer.pop_with_depth(self.min_depth);
        log::debug!("found {} points with required min depth", ready.len());

        // request download of blocks for confirmed points
        for point in ready {
            log::debug!("requesting block fetch for point {:?}", point);
            self.block_requests.send(point.clone())?;
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
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");
                self.event_writer.append_rollback_event(point)?;
            }
        }

        log_buffer_state(&self.chain_buffer);

        Ok(chainsync::Continuation::Proceed)
    }
}

pub(crate) fn fetch_blocks_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    input: Receiver<Point>,
) -> Result<(), Error> {
    let observer = Block2EventMapper(event_writer);
    let agent = blockfetch::OnDemandClient::initial(input.iter(), observer);
    let agent = run_agent(agent, &mut channel)?;
    log::debug!("blockfetch agent final state: {:?}", agent.state);

    Ok(())
}

fn observe_headers_forever(
    mut channel: StdChannel,
    event_writer: EventWriter,
    known_points: Option<Vec<Point>>,
    block_requests: SyncSender<Point>,
    min_depth: usize,
    finalize_config: Option<FinalizeConfig>,
) -> Result<(), AttemptError> {
    let observer = &mut ChainObserver {
        chain_buffer: Default::default(),
        min_depth,
        event_writer,
        block_requests,
        block_count: 0,
        finalize_config,
    };

    let agent = chainsync::HeaderConsumer::initial(known_points, observer);

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
    let versions = handshake::n2n::VersionTable::v6_and_above(magic);

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
    let mut cs_channel = plexer.use_channel(2);
    let bf_channel = plexer.use_channel(3);

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

    let (headers_tx, headers_rx) = std::sync::mpsc::sync_channel(100);

    let bf_writer = writer.clone();
    std::thread::spawn(move || {
        fetch_blocks_forever(bf_channel, bf_writer, headers_rx).expect("blockfetch loop failed");

        log::info!("block fetch thread ended");
    });

    // this will block
    observe_headers_forever(
        cs_channel,
        writer,
        known_points,
        headers_tx,
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
