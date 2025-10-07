use std::collections::HashMap;

use gasket::framework::*;
use pallas::interop::utxorpc::spec::sync::BlockRef;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;
use utxorpc::{CardanoSyncClient, ChainBlock, ClientBuilder, TipEvent};

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
    fn block_to_record(
        &self,
        stage: &Stage,
        block: &ChainBlock<utxorpc::spec::cardano::Block>,
    ) -> Result<(Point, Record), WorkerError> {
        let parsed = block.parsed.as_ref().ok_or(WorkerError::Panic)?;

        let record = if stage.config.use_parsed_blocks {
            Record::Cardano(cardano::Record::ParsedBlock(parsed.clone()))
        } else {
            Record::Cardano(cardano::Record::CborBlock(block.native.to_vec()))
        };

        let point = parsed
            .header
            .as_ref()
            .map(|h| Point::Specific(h.slot, h.hash.to_vec()))
            .ok_or(WorkerError::Panic)?;

        Ok((point, record))
    }

    async fn process_next(
        &self,
        stage: &mut Stage,
        unit: &TipEvent<utxorpc::Cardano>,
    ) -> Result<(), WorkerError> {
        match unit {
            TipEvent::Apply(block) => {
                let (point, record) = self.block_to_record(stage, block)?;

                let evt = ChainEvent::Apply(point.clone(), record);

                stage.output.send(evt.into()).await.or_panic()?;
                stage.chain_tip.set(point.slot_or_default() as i64);
            }
            TipEvent::Undo(block) => {
                let (point, record) = self.block_to_record(stage, block)?;

                let evt = ChainEvent::Undo(point.clone(), record);

                stage.output.send(evt.into()).await.or_panic()?;
                stage.chain_tip.set(point.slot_or_default() as i64);
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

        let mut builder = ClientBuilder::new()
            .uri(stage.config.url.as_str())
            .or_panic()?;

        for (key, value) in stage.config.metadata.iter() {
            builder = builder
                .metadata(key.to_string(), value.to_string())
                .or_panic()?;
        }

        let mut client = builder.build::<CardanoSyncClient>().await;

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
    pub url: String,
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub use_parsed_blocks: bool,
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
