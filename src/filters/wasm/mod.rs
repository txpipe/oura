//! A mapper with custom logic from a WASM module

use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;

use crate::framework::*;

#[derive(Default)]
struct Worker {
    msg_count: gasket::metrics::Counter,
    input: MapperInputPort,
    output: MapperOutputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("msg_count", &self.msg_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;
        self.output.send(msg)?;

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.0.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("mapper_noop"),
        );

        Ok(vec![worker_tether])
    }
}

#[derive(Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker::default();

        Ok(Bootstrapper(worker))
    }
}
