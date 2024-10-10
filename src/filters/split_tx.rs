//! A noop filter used as example and placeholder for other filters

use gasket::framework::*;
use serde::Deserialize;
use std::borrow::Cow;

use pallas::ledger::traverse as trv;

use crate::framework::*;

type CborTx<'a> = Cow<'a, [u8]>;
type CborUtxo<'a> = Cow<'a, [u8]>;

fn map_tx_to_utxo(cbor: CborTx) -> Result<Vec<(TxoRef, Option<CborUtxo>, Spent)>, WorkerError> {
    let tx = trv::MultiEraTx::decode(cbor.as_ref()).or_panic()?;

    let utxos: Vec<_> = tx
        .produces()
        .iter()
        .map(|(idx, utxo)| {
            (
                (tx.hash(), *idx as u32),
                Some(Cow::Owned(utxo.encode())),
                false,
            )
        })
        .collect();

    Ok(utxos)
}

#[derive(Default, Stage)]
#[stage(name = "filter-split-tx", unit = "ChainEvent", worker = "Worker")]
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

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let output = unit.clone().try_map_record_to_many(|r| match r {
        Record::CborTx(_, cbor) => {
            let out = map_tx_to_utxo(Cow::Borrowed(&cbor))?
                .into_iter()
                .map(|(txo, cbor, spent)| Record::CborUtxo(txo, cbor.map(|cbor| cbor.into()), spent))
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
