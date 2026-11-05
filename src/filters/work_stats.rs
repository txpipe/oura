//! A pass-through filter that tracks work stats and enforces a finalization
//! policy.
//!
//! Today its observable behavior is the finalization policy: it forwards every
//! event downstream unchanged while counting applied blocks, and once the
//! configured policy is reached it ends its stage, which gracefully stops the
//! whole daemon (gasket halts when any stage reaches `Ended`). Because it lives
//! in the filter chain it is decoupled from any particular source.
//!
//! It is framed as `work_stats` so it can grow into a home for richer
//! progress/throughput stats in the future; the metrics below are the seed of
//! that.

use gasket::framework::*;
use serde::Deserialize;
use tracing::info;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "filter-work-stats", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    finalize: FinalizeConfig,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    block_count: gasket::metrics::Counter,

    #[metric]
    latest_slot: gasket::metrics::Gauge,
}

#[derive(Default)]
pub struct Worker {
    blocks: u64,
    finished: bool,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Default::default())
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        // once the finalization policy has been reached we end the stage, which
        // signals the daemon to tear down the whole pipeline.
        if self.finished {
            return Ok(WorkSchedule::Done);
        }

        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        // forward downstream unchanged; this filter never mutates the stream.
        stage.output.send(unit.clone().into()).await.or_panic()?;

        // finalization is keyed on applied blocks, mirroring the v1 source
        // `max_block_quantity` semantics. Undo/Reset still advance the policy's
        // notion of the latest point but don't count towards the block total.
        let point = match unit {
            ChainEvent::Apply(point, _) => {
                self.blocks += 1;
                stage.block_count.inc(1);
                stage.latest_slot.set(point.slot_or_default() as i64);
                point.clone()
            }
            ChainEvent::Undo(point, _) => point.clone(),
            ChainEvent::Reset(point) => point.clone(),
        };

        stage.ops_count.inc(1);

        if should_finalize(&stage.finalize, &point, self.blocks) {
            info!(
                blocks = self.blocks,
                slot = point.slot_or_default(),
                "work-stats finalization policy reached, stopping pipeline"
            );
            self.finished = true;
        }

        Ok(())
    }
}

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub until_hash: Option<String>,

    #[serde(default)]
    pub max_block_slot: Option<u64>,

    #[serde(default)]
    pub max_block_quantity: Option<u64>,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            finalize: FinalizeConfig {
                until_hash: self.until_hash,
                max_block_slot: self.max_block_slot,
                max_block_quantity: self.max_block_quantity,
            },
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
            block_count: Default::default(),
            latest_slot: Default::default(),
        };

        Ok(stage)
    }
}
