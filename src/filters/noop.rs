//! A noop filter used as example and placeholder for other filters

use serde::Deserialize;

use crate::framework::*;

#[derive(Default)]
struct Worker {
    msg_count: gasket::metrics::Counter,
    input: FilterInputPort,
    output: FilterOutputPort,
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
    pub fn borrow_input_port(&mut self) -> &mut FilterInputPort {
        &mut self.0.input
    }

    pub fn borrow_output_port(&mut self) -> &mut FilterOutputPort {
        &mut self.0.output
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("filter_noop"),
        );

        Ok(Runtime { worker_tether })
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
