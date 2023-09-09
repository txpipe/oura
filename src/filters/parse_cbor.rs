//! A filter that turns raw cbor Tx into the corresponding parsed representation

use gasket::framework::*;
use serde::Deserialize;

use pallas::interop::utxorpc as interop;
use pallas::ledger::traverse as trv;
use utxorpc::proto::cardano::v1 as u5c;

use crate::framework::*;

fn map_cbor_to_u5c(cbor: &[u8]) -> Result<u5c::Tx, WorkerError> {
    let tx = trv::MultiEraTx::decode(trv::Era::Babbage, cbor)
        .or_else(|_| trv::MultiEraTx::decode(trv::Era::Alonzo, cbor))
        .or_else(|_| trv::MultiEraTx::decode(trv::Era::Byron, cbor))
        .or_panic()?;

    Ok(interop::map_tx(&tx))
}

#[derive(Default, Stage)]
#[stage(name = "filter-parse-cbor", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

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
            let tx = map_cbor_to_u5c(&cbor)?;
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
