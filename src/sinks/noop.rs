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
        stage.cursor.send(unit.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-noop", unit = "Point", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
