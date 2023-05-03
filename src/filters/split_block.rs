//! A noop filter used as example and placeholder for other filters

use gasket::framework::*;
use serde::Deserialize;
use std::borrow::Cow;

use pallas::ledger::traverse as trv;

use crate::framework::*;

type CborBlock<'a> = Cow<'a, [u8]>;
type CborTx<'a> = Cow<'a, [u8]>;

fn map_block_to_tx(cbor: CborBlock) -> Result<Vec<CborTx>, WorkerError> {
    let block = trv::MultiEraBlock::decode(cbor.as_ref()).or_panic()?;

    let txs: Vec<_> = block
        .txs()
        .iter()
        .map(|tx| tx.encode())
        .map(Cow::Owned)
        .collect();

    Ok(txs)
}

#[derive(Default, Stage)]
#[stage(name = "filter", unit = "ChainEvent", worker = "Worker")]
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
        Worker::default()
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let output = unit.clone().try_map_record_to_many(|r| match r {
        Record::CborBlock(cbor) => {
            let out = map_block_to_tx(Cow::Borrowed(&cbor))?
                .into_iter()
                .map(|tx| Record::CborTx(tx.into()))
                .collect();

            Ok(out)
        }
        x => Ok(vec![x]),
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
