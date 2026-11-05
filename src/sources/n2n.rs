use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, info, warn};

use pallas::ledger::traverse::MultiEraHeader;
use pallas::network::facades::PeerClient;
use pallas::network::miniprotocols::chainsync::{HeaderContent, NextResponse};
use pallas::network::miniprotocols::Point;

use crate::framework::*;

#[derive(Stage)]
#[stage(
    name = "source",
    unit = "Vec<NextResponse<HeaderContent>>",
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    chain: GenesisValues,

    intersect: IntersectConfig,

    breadcrumbs: Breadcrumbs,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,

    #[metric]
    current_slot: gasket::metrics::Gauge,

    #[metric]
    rollback_count: gasket::metrics::Counter,
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

async fn intersect_from_breadcrumbs(
    peer: &mut PeerClient,
    breadcrumbs: &Breadcrumbs,
) -> Result<(), WorkerError> {
    let (intersect, _) = peer
        .chainsync()
        .find_intersect(breadcrumbs.points())
        .await
        .or_restart()?;

    info!(?intersect, "intersected");

    Ok(())
}

/// How many blocks to fetch in one request while catching up.
const MAX_BATCH: usize = 50;

/// How far behind the tip, in slots, the source has to be to batch. Closer than
/// this, blocks are delivered as soon as their header arrives.
const MIN_BATCH_DISTANCE: u64 = 3600;

fn should_extend_batch(len: usize, slot: u64, tip_slot: u64) -> bool {
    len < MAX_BATCH && tip_slot.saturating_sub(slot) >= MIN_BATCH_DISTANCE
}

fn can_extend_batch(batch: &[NextResponse<HeaderContent>]) -> Result<bool, WorkerError> {
    let Some(NextResponse::RollForward(header, tip)) = batch.last() else {
        return Ok(false);
    };

    let slot = to_traverse(header)?.slot();

    Ok(should_extend_batch(
        batch.len(),
        slot,
        tip.0.slot_or_default(),
    ))
}

fn to_point(header: &HeaderContent) -> Result<Point, WorkerError> {
    let header = to_traverse(header)?;

    Ok(Point::Specific(header.slot(), header.hash().to_vec()))
}

pub struct Worker {
    peer_session: PeerClient,
}

impl Worker {
    /// Fetches every block in the run at once, before sending any, so a retry
    /// can't deliver a block twice.
    async fn process_forwards(
        &mut self,
        stage: &mut Stage,
        run: &[NextResponse<HeaderContent>],
    ) -> Result<(), WorkerError> {
        let mut points = Vec::with_capacity(run.len());

        for next in run {
            if let NextResponse::RollForward(header, _) = next {
                points.push(to_point(header)?);
            }
        }

        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            return Ok(());
        };

        debug!(?first, ?last, "fetching blocks");

        let blockfetch = self.peer_session.blockfetch();

        let blocks = if points.len() == 1 {
            vec![blockfetch.fetch_single(first.clone()).await.or_retry()?]
        } else {
            blockfetch
                .fetch_range((first.clone(), last.clone()))
                .await
                .or_retry()?
        };

        if blocks.len() != points.len() {
            warn!(
                expected = points.len(),
                received = blocks.len(),
                "peer returned an unexpected number of blocks"
            );
            return Err(WorkerError::Retry);
        }

        for ((point, block), next) in points.into_iter().zip(blocks).zip(run) {
            stage
                .output
                .send(ChainEvent::Apply(point.clone(), Record::CborBlock(block)).into())
                .await
                .or_panic()?;

            if let NextResponse::RollForward(_, tip) = next {
                stage.chain_tip.set(tip.0.slot_or_default() as i64);
            }

            stage.current_slot.set(point.slot_or_default() as i64);
            stage.breadcrumbs.track(point);
            stage.ops_count.inc(1);
        }

        Ok(())
    }

    async fn process_other(
        &mut self,
        stage: &mut Stage,
        next: &NextResponse<HeaderContent>,
    ) -> Result<(), WorkerError> {
        match next {
            NextResponse::RollForward(..) => Ok(()),
            NextResponse::RollBackward(point, tip) => {
                match &point {
                    Point::Origin => debug!("rollback to origin"),
                    Point::Specific(slot, _) => debug!(slot, "rollback"),
                };

                stage
                    .output
                    .send(ChainEvent::reset(point.clone()))
                    .await
                    .or_panic()?;

                stage.breadcrumbs.track(point.clone());

                stage.chain_tip.set(tip.0.slot_or_default() as i64);
                stage.current_slot.set(point.slot_or_default() as i64);
                stage.ops_count.inc(1);
                stage.rollback_count.inc(1);

                Ok(())
            }
            NextResponse::Await => {
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

        if stage.breadcrumbs.is_empty() {
            intersect_from_config(&mut peer_session, &stage.intersect).await?;
        } else {
            intersect_from_breadcrumbs(&mut peer_session, &stage.breadcrumbs).await?;
        }

        let worker = Self { peer_session };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<Vec<NextResponse<HeaderContent>>>, WorkerError> {
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

        let mut batch = vec![next];

        while client.has_agency() && can_extend_batch(&batch)? {
            batch.push(client.request_next().await.or_restart()?);
        }

        Ok(WorkSchedule::Unit(batch))
    }

    async fn execute(
        &mut self,
        unit: &Vec<NextResponse<HeaderContent>>,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        let forwards = unit
            .iter()
            .take_while(|next| matches!(next, NextResponse::RollForward(..)))
            .count();

        let (run, rest) = unit.split_at(forwards);

        self.process_forwards(stage, run).await?;

        for next in rest {
            self.process_other(stage, next).await?;
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            breadcrumbs: ctx.breadcrumbs.clone(),
            chain: ctx.chain.clone().into(),
            intersect: ctx.intersect.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            rollback_count: Default::default(),
            chain_tip: Default::default(),
            current_slot: Default::default(),
        };

        Ok(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_while_far_behind_the_tip() {
        assert!(should_extend_batch(1, 1000, 1000 + MIN_BATCH_DISTANCE));
    }

    #[test]
    fn stops_batching_near_the_tip() {
        assert!(!should_extend_batch(1, 1000, 1000 + MIN_BATCH_DISTANCE - 1));
        assert!(!should_extend_batch(1, 1000, 500));
    }

    #[test]
    fn stops_at_the_batch_limit() {
        assert!(!should_extend_batch(MAX_BATCH, 1000, u64::MAX));
    }

    #[test]
    fn nothing_extends_a_batch_after_a_rollback_or_await() {
        let rollback = NextResponse::RollBackward(
            Point::Origin,
            pallas::network::miniprotocols::chainsync::Tip(Point::Origin, 0),
        );

        assert!(!can_extend_batch(&[rollback]).unwrap());
        assert!(!can_extend_batch(&[NextResponse::Await]).unwrap());
    }
}
