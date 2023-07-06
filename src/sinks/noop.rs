//! A noop sink used as example and placeholder for other sinks

use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

#[derive(Default)]
pub struct Worker;

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Self)
    }

    async fn schedule(&mut self, stage: &mut Stage) -> Result<WorkSchedule<Point>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;

        let point = msg.payload.point().clone();
        Ok(WorkSchedule::Unit(point))
    }

    async fn execute(&mut self, unit: &Point, stage: &mut Stage) -> Result<(), WorkerError> {
        debug!(?unit, "message received");
        stage.ops_count.inc(1);

        stage.latest_block.set(unit.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(unit.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-noop", unit = "Point", worker = "Worker")]
pub struct Stage {
    cursor: Cursor,

    pub input: FilterInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(stage)
    }
}
