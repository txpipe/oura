//! A filter that turns raw cbor Tx into the corresponding parsed representation

use gasket::framework::*;
use serde::Deserialize;

use pallas::ledger::traverse as trv;
use utxorpc_spec_ledger::v1 as u5c;

use crate::framework::*;

fn from_traverse_tx(tx: &trv::MultiEraTx) -> u5c::Tx {
    u5c::Tx {
        inputs: tx
            .inputs()
            .iter()
            .map(|i| u5c::TxInput {
                tx_hash: i.hash().to_vec().into(),
                output_index: i.index() as u32,
                as_output: None,
            })
            .collect(),
        outputs: tx
            .outputs()
            .iter()
            .map(|o| u5c::TxOutput {
                address: o.address().map(|a| a.to_vec()).unwrap_or_default().into(),
                coin: o.lovelace_amount(),
                // TODO: this is wrong, we're crating a new item for each asset even if they share
                // the same policy id. We need to adjust Pallas' interface to make this mapping more
                // ergonomic.
                assets: o
                    .non_ada_assets()
                    .iter()
                    .map(|a| u5c::Multiasset {
                        policy_id: a.policy().map(|x| x.to_vec()).unwrap_or_default().into(),
                        assets: vec![u5c::Asset {
                            name: a.name().map(|x| x.to_vec()).unwrap_or_default().into(),
                            quantity: a.coin() as u64,
                        }],
                    })
                    .collect(),
                datum: None,
                datum_hash: Default::default(),
                script: None,
                redeemer: None,
            })
            .collect(),
        certificates: vec![],
        withdrawals: vec![],
        mint: vec![],
        reference_inputs: vec![],
        witnesses: u5c::WitnessSet {
            vkeywitness: vec![],
            script: vec![],
            plutus_datums: vec![],
        }
        .into(),
        collateral: u5c::Collateral {
            collateral: vec![],
            collateral_return: None,
            total_collateral: Default::default(),
        }
        .into(),
        fee: tx.fee().unwrap_or_default(),
        validity: u5c::TxValidity {
            start: tx.validity_start().unwrap_or_default(),
            ttl: tx.ttl().unwrap_or_default(),
        }
        .into(),
        successful: tx.is_valid(),
        auxiliary: u5c::AuxData {
            metadata: vec![],
            scripts: vec![],
        }
        .into(),
    }
}

fn map_cbor_to_u5c(cbor: &[u8]) -> Result<u5c::Tx, WorkerError> {
    let tx = trv::MultiEraTx::decode(trv::Era::Babbage, cbor)
        .or_else(|_| trv::MultiEraTx::decode(trv::Era::Alonzo, cbor))
        .or_else(|_| trv::MultiEraTx::decode(trv::Era::Byron, cbor))
        .or_panic()?;

    Ok(from_traverse_tx(&tx))
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
