//! A mapper that outputs events in JSON format

use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;

use crate::framework::*;

#[derive(Default)]
struct Worker {
    msg_count: gasket::metrics::Counter,
    input: MapperInputPort,
    output: MapperOutputPort,
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = ChainEvent;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("msg_count", &self.msg_count)
            .build()
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;

        Ok(gasket::runtime::WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        self.output.send(unit.clone().into()).await?;

        Ok(())
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
            Some("mapper_wasm"),
        );

        Ok(vec![worker_tether])
    }
}

#[derive(Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker::default();

        Ok(Bootstrapper(worker))
    }
}
