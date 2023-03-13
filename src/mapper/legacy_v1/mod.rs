//! A mapper that maintains schema-compatibility with Oura v1

mod babbage;
mod cip15;
mod cip25;
mod map;
mod prelude;

pub use prelude::*;
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

pub struct Runtime {
    worker_tether: gasket::runtime::Tether,
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &mut MapperInputPort {
        &mut self.0.input
    }

    pub fn borrow_output_port(&mut self) -> &mut MapperOutputPort {
        &mut self.0.output
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("mapper_legacy_v1"),
        );

        Ok(Runtime { worker_tether })
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub include_block_end_events: bool,

    #[serde(default)]
    pub include_transaction_details: bool,

    #[serde(default)]
    pub include_transaction_end_events: bool,

    #[serde(default)]
    pub include_block_details: bool,

    #[serde(default)]
    pub include_block_cbor: bool,

    #[serde(default)]
    pub include_byron_ebb: bool,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker::default();

        Ok(Bootstrapper(worker))
    }
}
