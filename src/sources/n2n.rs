use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, info};

use pallas::ledger::traverse::MultiEraHeader;
use pallas::network::facades::PeerClient;
use pallas::network::miniprotocols::chainsync::{self, HeaderContent, NextResponse};
use pallas::network::miniprotocols::Point;

use crate::framework::*;

#[derive(Stage)]
#[stage(
    name = "source-n2n",
    unit = "NextResponse<HeaderContent>",
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    chain: GenesisValues,

    intersect: IntersectConfig,

    cursor: Cursor,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,
}

fn to_traverse(header: &HeaderContent) -> Result<MultiEraHeader<'_>, WorkerError> {
    let out = match header.byron_prefix {
        Some((subtag, _)) => MultiEraHeader::decode(header.variant, Some(subtag), &header.cbor),
        None => MultiEraHeader::decode(header.variant, None, &header.cbor),
    };

    out.or_panic()
}

async fn intersect_from_config(
    peer: &mut PeerClient,
    intersect: &IntersectConfig,
) -> Result<(), WorkerError> {
    let chainsync = peer.chainsync();

    let intersect = match intersect {
        IntersectConfig::Origin => {
            info!("intersecting origin");
            chainsync.intersect_origin().await.or_restart()?.into()
        }
        IntersectConfig::Tip => {
            info!("intersecting tip");
            chainsync.intersect_tip().await.or_restart()?.into()
        }
        IntersectConfig::Point(..) | IntersectConfig::Breadcrumbs(..) => {
            info!("intersecting specific points");
            let points = intersect.points().unwrap_or_default();
            let (point, _) = chainsync.find_intersect(points).await.or_restart()?;
            point
        }
    };

    info!(?intersect, "intersected");

    Ok(())
}

async fn intersect_from_cursor(peer: &mut PeerClient, cursor: &Cursor) -> Result<(), WorkerError> {
    let points = cursor.clone_state();

    let (intersect, _) = peer
        .chainsync()
        .find_intersect(points.into())
        .await
        .or_restart()?;

    info!(?intersect, "intersected");

    Ok(())
}

pub struct Worker {
    peer_session: PeerClient,
}

impl Worker {
    async fn process_next(
        &mut self,
        stage: &mut Stage,
        next: &NextResponse<HeaderContent>,
    ) -> Result<(), WorkerError> {
        match next {
            NextResponse::RollForward(header, tip) => {
                let header = to_traverse(header).or_panic()?;
                let slot = header.slot();
                let hash = header.hash();

                debug!(slot, %hash, "chain sync roll forward");

                let block = self
                    .peer_session
                    .blockfetch()
                    .fetch_single(Point::Specific(slot, hash.to_vec()))
                    .await
                    .or_retry()?;

                let evt = ChainEvent::Apply(
                    pallas::network::miniprotocols::Point::Specific(slot, hash.to_vec()),
                    Record::CborBlock(block),
                );

                stage.output.send(evt.into()).await.or_panic()?;

                stage.chain_tip.set(tip.0.slot_or_default() as i64);

                Ok(())
            }
            chainsync::NextResponse::RollBackward(point, tip) => {
                match &point {
                    Point::Origin => debug!("rollback to origin"),
                    Point::Specific(slot, _) => debug!(slot, "rollback"),
                };

                stage
                    .output
                    .send(ChainEvent::reset(point.clone()).into())
                    .await
                    .or_panic()?;

                stage.chain_tip.set(tip.0.slot_or_default() as i64);

                Ok(())
            }
            chainsync::NextResponse::Await => {
                info!("chain-sync reached the tip of the chain");
                Ok(())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let peer_address = stage
            .config
            .peers
            .first()
            .cloned()
            .ok_or_else(|| Error::config("at least one upstream peer is required"))
            .or_panic()?;

        let mut peer_session = PeerClient::connect(&peer_address, stage.chain.magic)
            .await
            .or_retry()?;

        if stage.cursor.is_empty() {
            intersect_from_config(&mut peer_session, &stage.intersect).await?;
        } else {
            intersect_from_cursor(&mut peer_session, &stage.cursor).await?;
        }

        let worker = Self { peer_session };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<NextResponse<HeaderContent>>, WorkerError> {
        let client = self.peer_session.chainsync();

        let next = match client.has_agency() {
            true => {
                info!("requesting next block");
                client.request_next().await.or_restart()?
            }
            false => {
                info!("awaiting next block (blocking)");
                client.recv_while_must_reply().await.or_restart()?
            }
        };

        Ok(WorkSchedule::Unit(next))
    }

    async fn execute(
        &mut self,
        unit: &NextResponse<HeaderContent>,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        self.process_next(stage, unit).await
    }

    async fn teardown(&mut self) -> Result<(), WorkerError> {
        self.peer_session.abort();

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            chain: ctx.chain.clone().into(),
            intersect: ctx.intersect.clone(),
            cursor: ctx.cursor.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
