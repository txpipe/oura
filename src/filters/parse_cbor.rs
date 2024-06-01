//! A filter that turns raw cbor Tx into the corresponding parsed representation

use gasket::framework::*;
use serde::Deserialize;

use pallas::interop::utxorpc::{self as interop};
use pallas::ledger::traverse as trv;

use crate::framework::*;

#[derive(Clone, Default)]
struct NoOpContext;

impl interop::Context for NoOpContext {
    fn get_txo<'a>(
        &self,
        _tx_hash: pallas::crypto::hash::Hash<32>,
        _txo_index: u32,
    ) -> Option<trv::MultiEraOutput<'a>> {
        None
    }
}

#[derive(Default, Stage)]
#[stage(name = "filter-parse-cbor", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    mapper: interop::Mapper<NoOpContext>,

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

gasket::impl_mapper!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let output = unit.clone().try_map_record(|r| match r {
        Record::CborTx(cbor) => {
            let tx = trv::MultiEraTx::decode(&cbor).or_panic()?;
            let tx = stage.mapper.map_tx(&tx);
            Ok(Record::ParsedTx(tx))
        }
        x => Ok(x),
    })?;

    stage.ops_count.inc(1);

    output
});

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        Ok(Stage::default())
    }
}
