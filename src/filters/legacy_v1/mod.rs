//! A mapper that maintains schema-compatibility with Oura v1

mod cip15;
mod cip25;
mod crawl;
mod map;
mod prelude;

use gasket::{messaging::*, runtime::Tether};
use pallas::ledger::traverse::wellknown::GenesisValues;
use serde::Deserialize;

use crate::framework::*;
pub use prelude::*;

#[derive(Default)]
struct Worker {
    ops_count: gasket::metrics::Counter,
    config: Config,
    genesis: GenesisValues,
    error_policy: RuntimePolicy,
    input: MapperInputPort,
    output: MapperOutputPort,
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = ChainEvent;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;

        Ok(gasket::runtime::WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        let mut buffer = Vec::new();

        match unit {
            ChainEvent::Apply(point, Record::CborBlock(cbor)) => {
                let mut writer = EventWriter::new(
                    point.clone(),
                    self.output.clone(),
                    &self.config,
                    &self.genesis,
                    &self.error_policy,
                    &mut buffer,
                );

                writer.crawl_cbor(&cbor)?;
            }
            ChainEvent::Reset(point) => {
                let mut writer = EventWriter::new(
                    point.clone(),
                    self.output.clone(),
                    &self.config,
                    &self.genesis,
                    &self.error_policy,
                    &mut buffer,
                );

                writer.crawl_rollback(point.clone())?;
            }
            x => buffer.push(x.clone()),
        };

        for evt in buffer {
            self.output.send(evt.into()).await?;
        }

        self.ops_count.inc(1);

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
            Some("mapper"),
        );

        Ok(vec![worker_tether])
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
        let worker = Worker {
            config: self,
            genesis: ctx.chain.clone(),
            ..Default::default()
        };

        Ok(Bootstrapper(worker))
    }
}
