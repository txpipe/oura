//! A noop sink used as example and placeholder for other sinks

use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

struct Worker;

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = Point;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
        Ok(Self)
    }

    async fn schedule(
        &mut self,
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;

        let point = msg.payload.point().clone();
        Ok(WorkSchedule::Unit(point))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        debug!(?unit, "message received");
        stage.ops_count.inc(1);

        stage.latest_block.set(unit.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(unit.clone());

        Ok(())
    }
}

pub struct Stage {
    ops_count: gasket::metrics::Counter,
    latest_block: gasket::metrics::Gauge,
    cursor: Cursor,
    input: FilterInputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "sink"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
        registry.track_gauge("latest_block", &self.latest_block);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }
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
