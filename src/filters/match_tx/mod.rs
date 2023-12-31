use gasket::framework::*;
use pallas::{
    ledger::addresses::{Address, StakeAddress},
    network::miniprotocols::Point,
};
use serde::Deserialize;
use thiserror::Error;
use tracing::error;

use crate::framework::*;

mod address;
mod eval;

#[derive(Error)]
pub enum Error {
    Decoding,
    MissingData,
}

#[derive(Stage)]
#[stage(name = "match_tx", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    predicate: Predicate,

    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Worker {}
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let out = match unit {
        ChainEvent::Apply(point, record) => match record {
            Record::ParsedTx(tx) => {
                if stage.predicate.tx_match(point, tx)? {
                    Ok(Some(unit.to_owned()))
                } else {
                    Ok(None)
                }
            },
            _ => {
                error!("The MatchPattern filter is valid only with the ParsedTx record");
                Err(WorkerError::Panic)
            }
        },
        _ => Ok(Some(unit.to_owned()))
    }?;

    stage.ops_count.inc(1);

    out
});

#[derive(Deserialize, Clone, Debug)]
pub struct BlockPattern {
    pub slot_before: Option<u64>,
    pub slot_after: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    Block(BlockPattern),
    AnyOutputMatches(OutputPattern),
    AnyAddressMatches(AddressPattern),
    // WithdrawalMatches(AddressPattern),
    // CollateralMatches(AddressPattern),
    Not(Box<Predicate>),
    AnyOf(Vec<Predicate>),
    AllOf(Vec<Predicate>),
}

fn block_match(point: &Point, block_pattern: &BlockPattern) -> bool {
    if let Some(slot_after) = block_pattern.slot_after {
        if point.slot_or_default() <= slot_after {
            return false;
        }
    }

    if let Some(slot_before) = block_pattern.slot_before {
        if point.slot_or_default() >= slot_before {
            return false;
        }
    }

    true
}

fn output_match(tx: &ParsedTx, address_pattern: &AddressPattern) -> Result<bool, WorkerError> {
    if address_pattern.is_script.unwrap_or_default() {
        // TODO: validate inside script
        return Ok(false);
    }

    for output in tx.outputs.iter() {
        let address = Address::from_bytes(&output.address).or_panic()?;
        if !address.has_script() && address_pattern.address_match(&address)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn withdrawal_match(tx: &ParsedTx, address_pattern: &AddressPattern) -> Result<bool, WorkerError> {
    for withdrawal in tx.withdrawals.iter() {
        let address = Address::from_bytes(&withdrawal.reward_account).or_panic()?;
        if address_pattern.address_match(&address)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn collateral_match(tx: &ParsedTx, address_pattern: &AddressPattern) -> Result<bool, WorkerError> {
    if tx.collateral.is_some() {
        if let Some(collateral_return) = &tx.collateral.as_ref().unwrap().collateral_return {
            let address = Address::from_bytes(&collateral_return.address).or_panic()?;
            return address_pattern.address_match(&address);
        }
    }

    Ok(false)
}

#[derive(Deserialize)]
pub struct Config {
    pub predicate: Predicate,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            predicate: self.predicate,
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
