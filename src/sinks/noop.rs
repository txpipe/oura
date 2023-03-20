//! A noop filter used as example and placeholder for other filters

use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

#[derive(Default)]
struct Worker {
    ops_count: gasket::metrics::Counter,
    input: FilterInputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;
        debug!(?msg, "message received");
        self.ops_count.inc(1);

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: FilterInputAdapter) {
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
        let worker = Worker::default();

        Ok(Bootstrapper(worker))
    }
}
