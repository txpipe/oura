use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;

use crate::framework::*;

#[derive(Default)]
pub struct Worker {}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Default::default())
    }

    async fn schedule(&mut self, stage: &mut Stage) -> Result<WorkSchedule<Point>, WorkerError> {
        let msg = stage.track.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Point, stage: &mut Stage) -> Result<(), WorkerError> {
        stage.breadcrumbs.track(unit.clone());
        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "cursor", unit = "Point", worker = "Worker")]
pub struct Stage {
    breadcrumbs: Breadcrumbs,

    pub track: gasket::messaging::InputPort<Point>,

    #[metric]
    tracked_slot: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config;

impl Config {
    pub fn initial_load(&self) -> Result<Breadcrumbs, Error> {
        Ok(Breadcrumbs::new(30))
    }

    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            breadcrumbs: ctx.breadcrumbs.clone(),
            tracked_slot: Default::default(),
            track: Default::default(),
        };

        Ok(stage)
    }
}
