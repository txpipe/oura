//! A mapper that maintains schema-compatibility with Oura v1

mod cip15;
mod cip25;
mod crawl;
mod map;
mod prelude;

use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use pallas::ledger::traverse::wellknown::GenesisValues;
use serde::Deserialize;

use crate::framework::*;
pub use prelude::*;

#[derive(Default)]
struct Worker;

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = ChainEvent;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
        Ok(Self)
    }

    async fn schedule(
        &mut self,
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;

        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        let mut buffer = Vec::new();

        match unit {
            ChainEvent::Apply(point, Record::CborBlock(cbor)) => {
                let mut writer = EventWriter::new(
                    point.clone(),
                    stage.output.clone(),
                    &stage.config,
                    &stage.genesis,
                    &mut buffer,
                );

                writer.crawl_cbor(&cbor)?;
            }
            ChainEvent::Reset(point) => {
                let mut writer = EventWriter::new(
                    point.clone(),
                    stage.output.clone(),
                    &stage.config,
                    &stage.genesis,
                    &mut buffer,
                );

                writer.crawl_rollback(point.clone())?;
            }
            x => buffer.push(x.clone()),
        };

        for evt in buffer {
            stage.output.send(evt.into()).await.or_panic()?;
        }

        stage.ops_count.inc(1);

        Ok(())
    }
}

pub struct Stage {
    ops_count: gasket::metrics::Counter,
    config: Config,
    genesis: GenesisValues,
    retries: gasket::retries::Policy,
    input: MapperInputPort,
    output: MapperOutputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "filter"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

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
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            genesis: ctx.chain.clone(),
            retries: ctx.retries.clone(),
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
