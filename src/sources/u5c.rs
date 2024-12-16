use gasket::framework::*;
use pallas::interop::utxorpc::spec::sync::BlockRef;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;
use utxorpc::{CardanoSyncClient, ClientBuilder, TipEvent};

use crate::framework::*;

fn point_to_blockref(point: Point) -> Option<BlockRef> {
    match point {
        Point::Origin => None,
        Point::Specific(slot, hash) => Some(BlockRef {
            index: slot,
            hash: hash.into(),
        }),
    }
}

pub struct Worker {
    stream: utxorpc::LiveTip<utxorpc::Cardano>,
}

impl Worker {
    async fn process_next(
        &self,
        stage: &mut Stage,
        unit: &TipEvent<utxorpc::Cardano>,
    ) -> Result<(), WorkerError> {
        match unit {
            TipEvent::Apply(block) => {
                if let Some(block) = &block.parsed {
                    let header = block.header.as_ref().unwrap();

                    let block = block.body.as_ref().unwrap();

                    for tx in block.tx.clone() {
                        let evt = ChainEvent::Apply(
                            Point::Specific(header.slot, header.hash.to_vec()),
                            Record::ParsedTx(tx),
                        );

                        stage.output.send(evt.into()).await.or_panic()?;
                        stage.chain_tip.set(header.slot as i64);
                    }
                }
            }
            TipEvent::Undo(block) => {
                if let Some(block) = &block.parsed {
                    let header = block.header.as_ref().unwrap();

                    let block = block.body.as_ref().unwrap();

                    for tx in block.tx.clone() {
                        let evt = ChainEvent::Undo(
                            Point::Specific(header.slot, header.hash.to_vec()),
                            Record::ParsedTx(tx),
                        );

                        stage.output.send(evt.into()).await.or_panic()?;
                        stage.chain_tip.set(header.slot as i64);
                    }
                }
            }
            TipEvent::Reset(block) => {
                stage
                    .output
                    .send(ChainEvent::Reset(Point::new(block.index, block.hash.to_vec())).into())
                    .await
                    .or_panic()?;

                stage.chain_tip.set(block.index as i64);
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let mut client = ClientBuilder::new()
            .uri(stage.config.url.as_str())
            .or_panic()?
            .build::<CardanoSyncClient>()
            .await;

        let intersect: Vec<_> = if stage.breadcrumbs.is_empty() {
            stage.intersect.points().unwrap_or_default()
        } else {
            stage.breadcrumbs.points()
        };

        let intersect = intersect
            .into_iter()
            .filter_map(point_to_blockref)
            .collect::<Vec<_>>()
            .pop();

        let stream = client
            .follow_tip(intersect.into_iter().collect())
            .await
            .or_restart()?;

        Ok(Self { stream })
    }

    async fn schedule(
        &mut self,
        _: &mut Stage,
    ) -> Result<WorkSchedule<TipEvent<utxorpc::Cardano>>, WorkerError> {
        let event = self.stream.event().await.or_restart()?;

        Ok(WorkSchedule::Unit(event))
    }

    async fn execute(
        &mut self,
        unit: &TipEvent<utxorpc::Cardano>,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        self.process_next(stage, unit).await.or_retry()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(
    name = "source-utxorpc",
    unit = "TipEvent<utxorpc::Cardano>",
    worker = "Worker"
)]
pub struct Stage {
    config: Config,
    breadcrumbs: Breadcrumbs,
    intersect: IntersectConfig,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,

    #[metric]
    current_slot: gasket::metrics::Gauge,
}

#[derive(Deserialize)]
pub struct Config {
    url: String,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            breadcrumbs: ctx.breadcrumbs.clone(),
            intersect: ctx.intersect.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
            current_slot: Default::default(),
        };

        Ok(stage)
    }
}
