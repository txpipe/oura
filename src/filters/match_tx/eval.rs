use std::ops::{Add, AddAssign};

use crate::framework::*;

use super::Predicate;

#[derive(Clone, Copy)]
enum MatchOutcome {
    Positive,
    Negative,
    Uncertain,
}

impl Add for MatchOutcome {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // one positive is enough
            (MatchOutcome::Positive, _) => MatchOutcome::Positive,
            (_, MatchOutcome::Positive) => MatchOutcome::Positive,

            // if there's any uncertainty, return uncertain
            (_, MatchOutcome::Uncertain) => MatchOutcome::Uncertain,
            (MatchOutcome::Uncertain, _) => MatchOutcome::Uncertain,

            // default to negative
            _ => MatchOutcome::Negative,
        }
    }
}

impl MatchOutcome {
    fn fold_any_of(outcomes: impl Iterator<Item = Self>) -> Self {
        let mut folded = MatchOutcome::Negative;

        for item in outcomes {
            if item == Self::Positive {
                return Self::Positive;
            }

            folded = folded + item;
        }

        folded
    }

    fn fold_all_of(outcomes: impl Iterator<Item = Self>) -> Self {
        for item in outcomes {
            if item != Self::Positive {
                return item;
            }
        }

        Self::Positive
    }
}

fn eval_tx(tx: &ParsedTx, predicate: Predicate) -> MatchOutcome {
    match predicate {
        Predicate::Block(block_pattern) => Ok(block_match(point, block_pattern)),
        Predicate::OutputAddress(address_pattern) => Ok(output_match(tx, address_pattern)?),
        Predicate::WithdrawalAddress(address_pattern) => Ok(withdrawal_match(tx, address_pattern)?),
        Predicate::CollateralAddress(address_pattern) => Ok(collateral_match(tx, address_pattern)?),
        Predicate::Not(x) => Ok(!x.tx_match(point, tx)?),
        Predicate::AnyOf(x) => {
            let o = x.iter().map(|x| eval_tx(tx, x));
            fold_any_of(o)
        }
        Predicate::AllOf(x) => {
            let o = x.iter().map(|x| eval_tx(tx, x));
            fold_all_of(o)
        }
    }
}

fn eval_block(block: &ParsedBlock, predicate: &Predicate) -> MatchOutcome {
    let outcomes = block
        .body
        .unwrap_or_default()
        .tx
        .iter()
        .map(|tx| eval_tx(tx, predicate));
}

pub fn eval(record: &Record, predicate: &Predicate) -> MatchOutcome {
    match record {
        Record::ParsedTx(x) => eval_tx(x, predicate),
        Record::ParsedBlock(x) => eval_block(x, predicate),
        _ => MatchOutcome::Uncertain,
    }
}
