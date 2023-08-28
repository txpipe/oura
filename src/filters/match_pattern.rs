use gasket::framework::*;
use pallas::{
    ledger::addresses::{Address, StakeAddress},
    network::miniprotocols::Point,
};
use serde::Deserialize;
use tracing::error;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "filter-match-pattern", unit = "ChainEvent", worker = "Worker")]
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
pub enum AddressPatternValue {
    ExactHex(String),
    ExactBech32(String),
    PaymentHex(String),
    PaymentBech32(String),
    StakeHex(String),
    StakeBech32(String),
}

#[derive(Deserialize, Clone, Debug)]
pub struct AddressPattern {
    pub value: AddressPatternValue,
    pub is_script: Option<bool>,
}
impl AddressPattern {
    fn address_match(&self, address: &Address) -> Result<bool, WorkerError> {
        match address {
            Address::Byron(addr) => match &self.value {
                AddressPatternValue::ExactHex(exact_hex) => Ok(addr.to_hex().eq(exact_hex)),
                AddressPatternValue::PaymentHex(payment_hex) => Ok(addr.to_hex().eq(payment_hex)),
                _ => Ok(false),
            },
            Address::Shelley(addr) => match &self.value {
                AddressPatternValue::ExactHex(exact_hex) => Ok(addr.to_hex().eq(exact_hex)),
                AddressPatternValue::ExactBech32(exact_bech32) => {
                    Ok(addr.to_bech32().or_panic()?.eq(exact_bech32))
                }
                AddressPatternValue::PaymentHex(payment_hex) => {
                    Ok(addr.payment().to_hex().eq(payment_hex))
                }
                AddressPatternValue::PaymentBech32(payment_bech32) => {
                    Ok(addr.payment().to_bech32().or_panic()?.eq(payment_bech32))
                }
                AddressPatternValue::StakeHex(stake_hex) => {
                    if addr.delegation().as_hash().is_none() {
                        return Ok(false);
                    }

                    let stake_address: StakeAddress = addr.clone().try_into().or_panic()?;
                    Ok(stake_address.to_hex().eq(stake_hex))
                }
                AddressPatternValue::StakeBech32(stake_bech32) => {
                    if addr.delegation().as_hash().is_none() {
                        return Ok(false);
                    }

                    let stake_address: StakeAddress = addr.clone().try_into().or_panic()?;
                    Ok(stake_address.to_bech32().or_panic()?.eq(stake_bech32))
                }
            },
            Address::Stake(stake_address) => match &self.value {
                AddressPatternValue::StakeHex(stake_hex) => {
                    Ok(stake_address.to_hex().eq(stake_hex))
                }
                AddressPatternValue::StakeBech32(stake_bech32) => {
                    Ok(stake_address.to_bech32().or_panic()?.eq(stake_bech32))
                }
                _ => Ok(false),
            },
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockPattern {
    pub slot_before: Option<u64>,
    pub slot_after: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    Block(BlockPattern),
    OutputAddress(AddressPattern),
    WithdrawalAddress(AddressPattern),
    CollateralAddress(AddressPattern),
}

impl Predicate {
    fn tx_match(&self, point: &Point, tx: &ParsedTx) -> Result<bool, WorkerError> {
        match self {
            Predicate::Block(block_pattern) => Ok(block_match(point, block_pattern)),
            Predicate::OutputAddress(address_pattern) => Ok(output_match(tx, address_pattern)?),
            Predicate::WithdrawalAddress(address_pattern) => {
                Ok(withdrawal_match(tx, address_pattern)?)
            }
            Predicate::CollateralAddress(address_pattern) => {
                Ok(collateral_match(tx, address_pattern)?)
            }
        }
    }
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
