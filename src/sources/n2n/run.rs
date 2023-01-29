use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use pallas::network::{
    miniprotocols::{blockfetch, chainsync, handshake, Point, MAINNET_MAGIC},
    multiplexer::StdChannel,
};

use std::sync::mpsc::{Receiver, SyncSender};

use crate::{
    mapper::EventWriter,
    pipelining::StageSender,
    sources::{
        intersect_starting_point, setup_multiplexer, should_finalize, unknown_block_to_events,
        FinalizeConfig,
    },
    utils::{retry, Utils},
    Error,
};

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

enum Continuation {
    Proceed,
    DropOut,
}

impl ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::HeaderContent,
        tip: &chainsync::Tip,
    ) -> Result<Continuation, Error> {
        // parse the header and extract the point of the chain

        let header = pallas::ledger::traverse::MultiEraHeader::decode(
            content.variant,
            content.byron_prefix.map(|x| x.0),
            &content.cbor,
        )?;

        let point = Point::Specific(header.slot(), header.hash().to_vec());

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
                return Ok(Continuation::DropOut);
            }
        }

        log_buffer_state(&self.chain_buffer);

        // notify chain tip to the pipeline metrics
        self.event_writer.utils.track_chain_tip(tip.1);

        Ok(Continuation::Proceed)
    }

    fn on_rollback(&mut self, point: &Point) -> Result<(), Error> {
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

        Ok(())
    }

    fn on_next_message(
        &mut self,
        msg: chainsync::NextResponse<chainsync::HeaderContent>,
        client: &mut chainsync::N2NClient<StdChannel>,
    ) -> Result<Continuation, AttemptError> {
        match msg {
            chainsync::NextResponse::RollForward(c, t) => match self.on_roll_forward(c, &t) {
                Ok(x) => Ok(x),
                Err(err) => Err(AttemptError::Other(err)),
            },
            chainsync::NextResponse::RollBackward(x, _) => match self.on_rollback(&x) {
                Ok(_) => Ok(Continuation::Proceed),
                Err(err) => Err(AttemptError::Other(err)),
            },
            chainsync::NextResponse::Await => {
                let next = client
                    .recv_while_must_reply()
                    .map_err(|x| AttemptError::Recoverable(x.into()))?;

                self.on_next_message(next, client)
            }
        }
    }
}

pub(crate) fn fetch_blocks_forever(
    mut client: blockfetch::Client<StdChannel>,
    event_writer: EventWriter,
    input: Receiver<Point>,
) -> Result<(), Error> {
    for point in input {
        let body = client.fetch_single(point.clone())?;

        unknown_block_to_events(&event_writer, &body)?;

        log::debug!("blockfetch succeeded: {:?}", point);
    }

    Ok(())
}

fn observe_headers_forever(
    mut client: chainsync::N2NClient<StdChannel>,
    event_writer: EventWriter,
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

    loop {
        match client.request_next() {
            Ok(next) => match observer.on_next_message(next, &mut client) {
                Ok(Continuation::Proceed) => (),
                Ok(Continuation::DropOut) => break Ok(()),
                Err(err) => break Err(err),
            },
            Err(err) => break Err(AttemptError::Recoverable(err.into())),
        }
    }
}

#[derive(Debug)]
enum AttemptError {
    Recoverable(Error),
    Other(Error),
}

fn do_handshake(channel: StdChannel, magic: u64) -> Result<(), AttemptError> {
    let mut client = handshake::N2NClient::new(channel);
    let versions = handshake::n2n::VersionTable::v4_and_above(magic);

    match client.handshake(versions) {
        Ok(confirmation) => match confirmation {
            handshake::Confirmation::Accepted(_, _) => Ok(()),
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

    let hs_channel = plexer.use_channel(0);
    let cs_channel = plexer.use_channel(2);
    let bf_channel = plexer.use_channel(3);

    plexer.muxer.spawn();
    plexer.demuxer.spawn();

    do_handshake(hs_channel, magic)?;

    let mut cs_client = chainsync::N2NClient::new(cs_channel);

    let intersection = intersect_starting_point(
        &mut cs_client,
        &config.intersect,
        #[allow(deprecated)]
        &config.since,
        &utils,
    )
    .map_err(|err| AttemptError::Recoverable(err))?;

    if intersection.is_none() {
        return Err(AttemptError::Other(
            "Can't find chain intersection point".into(),
        ));
    }

    log::info!("starting chain sync from: {:?}", &intersection);

    let bf_client = blockfetch::Client::new(bf_channel);
    let writer = EventWriter::new(output_tx.clone(), utils, config.mapper.clone());

    let (headers_tx, headers_rx) = std::sync::mpsc::sync_channel(100);

    let bf_writer = writer.clone();
    std::thread::spawn(move || {
        fetch_blocks_forever(bf_client, bf_writer, headers_rx).expect("blockfetch loop failed");

        log::info!("block fetch thread ended");
    });

    // this will block
    observe_headers_forever(
        cs_client,
        writer,
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
                .unwrap_or_else(|| Duration::from_secs(60)),
        },
    )
}
