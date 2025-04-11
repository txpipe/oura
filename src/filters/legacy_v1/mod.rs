//! A mapper that maintains schema-compatibility with Oura v1

mod cip15;
mod cip25;
mod crawl;
mod map;
mod prelude;

use gasket::framework::*;
use pallas::ledger::traverse::wellknown::GenesisValues;
use serde::Deserialize;

use crate::framework::*;
pub use prelude::*;

#[derive(Stage)]
#[stage(name = "filter-legacy", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    genesis: GenesisValues,

    pub input: MapperInputPort,
    pub output: MapperOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Self
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let mut buffer = Vec::new();

    match unit {
        ChainEvent::Apply(point, Record::CborBlock(cbor)) => {
            let mut writer = EventWriter::new(
                point.clone(),
                &stage.output,
                &stage.config,
                &stage.genesis,
                &mut buffer,
            );

            writer.crawl_cbor(cbor)?;
        }
        ChainEvent::Reset(point) => {
            let mut writer = EventWriter::new(
                point.clone(),
                &stage.output,
                &stage.config,
                &stage.genesis,
                &mut buffer,
            );

            writer.crawl_rollback(point.clone())?;
        }
        x => buffer.push(x.clone()),
    };

    stage.ops_count.inc(1);

    buffer
});

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
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            genesis: ctx.chain.clone().into(),
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
