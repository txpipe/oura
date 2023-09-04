use pallas::ledger::addresses::Address;
use thiserror::Error;
use utxorpc::proto::cardano::v1::{Asset, Multiasset, PlutusData, Tx, TxInput, TxOutput};

use crate::framework::Record;

use super::{
    AddressPattern, AssetPattern, DatumPattern, InputPattern, OutputPattern, QuantityPattern,
    TxPredicate, UtxoRefPattern,
};

fn eval_quantity_matches(value: u64, pattern: &QuantityPattern) -> EvalResult {
    let eval = match pattern {
        QuantityPattern::Equals(expected) => value.eq(expected),
        QuantityPattern::RangeInclusive(a, b) => value.ge(a) && value.le(b),
        QuantityPattern::Greater(a) => value.gt(a),
        QuantityPattern::GreaterOrEqual(_) => value.ge(a),
        QuantityPattern::Lower(a) => value.lt(a),
        QuantityPattern::LowerOrEqual(b) => value.le(b),
    };

    Ok(eval)
}

fn eval_block_matches(point: &Point, pattern: &BlockPattern) -> EvalResult {
    if let Some(slot_after) = block_pattern.slot_after {
        if point.slot_or_default() <= slot_after {
            return Ok(false);
        }
    }

    if let Some(slot_before) = block_pattern.slot_before {
        if point.slot_or_default() >= slot_before {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_address_matches(addr: &[u8], pattern: &AddressPattern) -> EvalResult {
    let addr =
        Address::from_bytes(addr).map_err(|_| Error::inconclusive("can't parse address bytes"))?;

    match (pattern, addr) {
        (AddressPattern::Exact(expected), _) => address.eq(expected),
        (AddressPattern::Payment(expected), Address::Shelley(shelley)) => {
            Ok(shelley.payment().eq(expected))
        }
        (AddressPattern::Delegation(expected), Address::Shelley(shelley)) => {
            Ok(shelley.delegation().eq(expected))
        }
        _ => Ok(false),
    }
}

fn eval_datum_matches(datum_hash: &[u8], pattern: &DatumPattern) -> EvalResult {
    if let Some(expected) = pattern.hash {
        let eval = datum_hash.eq(&expected);

        if !eval {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_asset_matches(policy: &[u8], asset: &Asset, pattern: &AssetPattern) -> EvalResult {
    if Some(pattern) = &pattern.policy {
        let eval = policy.eq(&pattern);

        if !eval {
            return Ok(false);
        }
    }

    if Some(pattern) = &pattern.quantity {
        let eval = eval_quantity_matches(asset.output_coin, pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_some_asset_matches(assets: &[Multiasset], pattern: &AssetPattern) -> EvalResult {
    for multiasset in assets.iter() {
        for asset in multiasset.assets.iter() {
            let eval = eval_asset_matches(&multiasset.policy_id, &asset, pattern)?;

            if eval {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn eval_output_matches(output: &TxOutput, pattern: &OutputPattern) -> EvalResult {
    if let Some(pattern) = &pattern.to {
        let eval = eval_address_matches(&output.address, &pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.datum {
        let eval = eval_datum_matches(&output.datum_hash, &pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.assets {
        let eval = eval_some_asset_matches(&output.assets, patter)?;

        if !eval {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_some_output_matches(tx: &Tx, pattern: &OutputPattern) -> Result<bool, WorkerError> {
    for output in tx.outputs.iter() {
        let eval = eval_output_matches(output, pattern)?;

        if eval {
            return Ok(true);
        }
    }

    Ok(false)
}

fn eval_some_withdrawal_matches(tx: &Tx, pattern: &OutputPattern) -> EvalResult {
    for withdrawal in tx.withdrawals.iter() {
        let eval = eval_output_matches(withdrawal, pattern)?;

        if eval {
            return Ok(true);
        }
    }

    Ok(false)
}

fn eval_some_collateral_matches(tx: &Tx, pattern: &InputPattern) -> EvalResult {
    if let Some(collateral) = &tx.collateral {
        for input in collateral.collateral.iter() {
            let eval = eval_input_matches(input, pattern)?;

            if eval {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn eval_collateral_return_matches(tx: &Tx, pattern: &OutputPattern) -> EvalResult {
    if let Some(collateral) = &tx.collateral {
        if let Some(return_) = collateral.collateral_return {
            let eval = eval_output_matches(&return_, pattern)?;

            if eval {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn eval_input_utxoref_matches(input: &TxInput, pattern: &UtxoRefPattern) -> EvalResult {
    if let Some(pattern) = &pattern.tx_hash {
        let eval = input.tx_hash.as_ref().eq(&pattern);

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.output_idx {
        let eval = input.output_index.eq(pattern);

        if !eval {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_input_matches(input: &TxInput, pattern: &InputPattern) -> EvalResult {
    if let Some(pattern) = pattern.utxo {
        let eval = eval_input_utxoref_matches(input, &pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.from {
        let output = input.as_output.ok_or(Error::no_input_ref())?;

        let eval = eval_address_matches(&output.address, &pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.datum {
        let output = input.as_output.ok_or(Error::no_input_ref())?;

        let eval = eval_datum_matches(&output.datum_hash, &pattern)?;

        if !eval {
            return Ok(false);
        }
    }

    if let Some(pattern) = &pattern.assets {
        let output = input.as_output.ok_or(Error::no_input_ref())?;

        let eval = eval_some_asset_matches(&output.assets, patter)?;

        if !eval {
            return Ok(false);
        }
    }

    Ok(true)
}

fn eval_some_input_matches(tx: &Tx, pattern: &InputPattern) -> EvalResult {
    for input in tx.inputs.iter() {
        let eval = eval_input_matches(input, pattern)?;

        if eval {
            return Ok(true);
        }
    }

    Ok(false)
}

fn eval_some_input_address_matches(tx: &Tx, pattern: &AddressPattern) -> EvalResult {
    for input in tx.inputs.iter() {
        let output = input.as_output.ok_or(Error::no_input_ref())?;

        let eval = eval_address_matches(&output.address, pattern)?;

        if eval {
            return Ok(true);
        }
    }

    Ok(false)
}

fn eval_some_input_asset_matches(tx: &Tx, pattern: &AssetPattern) -> EvalResult {
    for input in tx.inputs.iter() {
        let output = input.as_output.ok_or(Error::no_input_ref())?;

        for multiasset in output.assets {
            let eval = eval_asset_matches(&multiasset.policy_id, asset, pattern)?;

            if eval {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn eval_some_output_address_matches(tx: &Tx, pattern: &AddressPattern) -> EvalResult {
    for output in tx.outputs.iter() {
        let eval = eval_address_matches(&output.address, pattern)?;

        if eval {
            return Ok(true);
        }
    }

    Ok(false)
}

fn eval_tx_any_of(predicates: &[TxPredicate], point: &Point, tx: &Tx) -> EvalResult {
    for p in x {
        if eval_tx(p, point, tx)? {
            return Ok(true);
        };
    }

    Ok(false)
}

fn eval_tx_all_of(predicates: &[TxPredicate], point: &Point, tx: &Tx) -> EvalResult {
    for p in x {
        if !p.tx_match(point, tx)? {
            return Ok(false);
        };
    }

    Ok(true)
}

#[inline]
fn eval_tx_not(predicate: &TxPredicate, point: &Point, tx: &Tx) -> EvalResult {
    let not = !eval_tx(predicate, point, tx)?;

    Ok(not)
}

fn eval_tx(predicate: &TxPredicate, point: &Point, tx: &Tx) -> EvalResult {
    match predicate {
        TxPredicate::AnyOf(x) => eval_tx_any_of(x, point, tx),
        TxPredicate::AllOf(x) => eval_tx_all_of(x, point, tx),
        TxPredicate::Not(x) => eval_tx_not(x, point, tx),
        TxPredicate::HashEquals(_) => todo!(),
        TxPredicate::IsValid(_) => todo!(),
        TxPredicate::BlockMatches(pattern) => eval_block_matches(point, pattern),
        TxPredicate::SomeInputMatches(x) => eval_some_input_matches(tx, x),
        TxPredicate::TotalInputAssetsMatch(_) => todo!(),
        TxPredicate::SomeInputAddressMatches(x) => eval_some_input_address_matches(tx, x),
        TxPredicate::SomeInputAssetMatches(p) => eval_some_input_asset_matches(tx, p),
        TxPredicate::SomeInputDatumMatches(_) => todo!(),
        TxPredicate::TotalOutputAssetsMatch(_) => todo!(),
        TxPredicate::SomeOutputMatches(p) => eval_some_output_matches(tx, p),
        TxPredicate::SomeOutputAddressMatches(p) => eval_some_output_address_matches(tx, p),
        TxPredicate::SomeOutputDatumMatches(_) => todo!(),
        TxPredicate::SomeOutputAssetMatches(_) => todo!(),
        TxPredicate::SomeMintedAssetMatches(_) => todo!(),
        TxPredicate::SomeBurnedAssetMatches(_) => todo!(),
        TxPredicate::SomeMetadataMatches(_) => todo!(),
        TxPredicate::SomeCollateralMatches(x) => eval_some_collateral_matches(tx, x),
        TxPredicate::CollateralReturnMatches(x) => eval_collateral_return_matches(tx, x),
        TxPredicate::TotalCollateralMatches(x) => eval_total_collateral_matches(tx, x),
        TxPredicate::SomeWithdrawalMatches(x) => eval_some_withdrawal_matches(tx, x),
        TxPredicate::SomeAddressMatches(_) => todo!(),
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("predicate evaluation is inconclusive {}")]
    Inconclusive(String),
}

impl Error {
    pub fn inconclusive(msg: impl Into<String>) -> Self {
        Self::inconclusive(msg.into())
    }

    pub fn no_input_ref() -> Self {
        Self::inconclusive("no UTxO data for input ref")
    }
}

pub type EvalResult = Result<bool, Error>;

pub fn eval(predicate: &TxPredicate, point: &Point, record: Record) -> EvalResult {
    match record {
        Record::ParsedTx(tx) => eval_tx(predicate, point, &tx),
        _ => Err(Error::inconclusive(
            "we only know how to evaluate parsed transaction records",
        )),
    }
}
