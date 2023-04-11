//! A noop filter used as example and placeholder for other filters

use gasket::{messaging::*, runtime::Tether};
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

struct Worker {
    ops_count: gasket::metrics::Counter,
    latest_block: gasket::metrics::Gauge,
    cursor: Cursor,
    input: FilterInputPort,
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = Point;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .with_gauge("latest_block", &self.latest_block)
            .build()
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;

        let point = msg.payload.point().clone();
        Ok(gasket::runtime::WorkSchedule::Unit(point))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        debug!(?unit, "message received");
        self.ops_count.inc(1);

        self.latest_block.set(unit.slot_or_default() as i64);
        self.cursor.add_breadcrumb(unit.clone());

        Ok(())
    }
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether =
            gasket::runtime::spawn_stage(self.0, gasket::runtime::Policy::default(), Some("sink"));

        Ok(vec![worker_tether])
    }
}

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker {
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(Bootstrapper(worker))
    }
}
