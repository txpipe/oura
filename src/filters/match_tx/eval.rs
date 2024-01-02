use serde::Deserialize;
use utxorpc::proto::cardano::v1::{Multiasset, TxOutput};

use crate::framework::*;

use super::address::AddressPattern;

#[derive(Clone, Copy, PartialEq)]
pub enum MatchOutcome {
    Positive,
    Negative,
    Uncertain,
}

impl core::ops::Not for MatchOutcome {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            MatchOutcome::Positive => MatchOutcome::Negative,
            MatchOutcome::Negative => MatchOutcome::Positive,
            MatchOutcome::Uncertain => MatchOutcome::Uncertain,
        }
    }
}

impl core::ops::AddAssign for MatchOutcome {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl core::ops::Add for MatchOutcome {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // one positive is enough
            (MatchOutcome::Positive, _) => MatchOutcome::Positive,
            (_, MatchOutcome::Positive) => MatchOutcome::Positive,

            // if not positive, check uncertainty
            (_, MatchOutcome::Uncertain) => MatchOutcome::Uncertain,
            (MatchOutcome::Uncertain, _) => MatchOutcome::Uncertain,

            // default to negative
            _ => MatchOutcome::Negative,
        }
    }
}

impl MatchOutcome {
    pub fn fold_any_of(outcomes: impl Iterator<Item = Self>) -> Self {
        let mut folded = MatchOutcome::Negative;

        for item in outcomes {
            if item == Self::Positive {
                return Self::Positive;
            }

            folded = folded + item;
        }

        folded
    }

    pub fn fold_all_of(outcomes: impl Iterator<Item = Self>) -> Self {
        for item in outcomes {
            if item != Self::Positive {
                return item;
            }
        }

        Self::Positive
    }

    pub fn if_true(value: bool) -> Self {
        if value {
            Self::Positive
        } else {
            Self::Negative
        }
    }

    pub fn if_false(value: bool) -> Self {
        if value {
            Self::Negative
        } else {
            Self::Positive
        }
    }

    pub fn if_equal<T>(a: &T, b: &T) -> Self
    where
        T: PartialEq + ?Sized,
    {
        Self::if_true(a.eq(b))
    }
}

pub trait PatternOf<S: ?Sized> {
    fn is_match(&self, subject: &S) -> MatchOutcome;

    fn is_any_match<'a, I>(&self, iter: I) -> MatchOutcome
    where
        I: Iterator<Item = &'a S>,
        S: 'a,
    {
        let outcomes = iter.map(|x| self.is_match(x));
        MatchOutcome::fold_any_of(outcomes)
    }
}

impl<S, P> PatternOf<S> for Option<P>
where
    P: PatternOf<S>,
{
    fn is_match(&self, subject: &S) -> MatchOutcome {
        match self {
            Some(x) => x.is_match(subject),
            // the absence of a pattern matches everything
            None => MatchOutcome::Positive,
        }
    }
}

impl PatternOf<Vec<u8>> for Vec<u8> {
    fn is_match(&self, subject: &Vec<u8>) -> MatchOutcome {
        MatchOutcome::if_equal(self, subject)
    }
}

impl PatternOf<bool> for bool {
    fn is_match(&self, subject: &bool) -> MatchOutcome {
        MatchOutcome::if_equal(self, subject)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum CoinPattern {
    Exact(u64),
    Gte(u64),
    Lte(u64),
    Between(u64, u64),
}

impl PatternOf<u64> for CoinPattern {
    fn is_match(&self, subject: &u64) -> MatchOutcome {
        match self {
            CoinPattern::Exact(x) => MatchOutcome::if_true(subject == x),
            CoinPattern::Gte(x) => MatchOutcome::if_true(subject >= x),
            CoinPattern::Lte(x) => MatchOutcome::if_true(subject <= x),
            CoinPattern::Between(a, b) => MatchOutcome::if_true(subject >= a && subject <= b),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum AsciiPattern {
    Exact(String),
    // TODO: Regex
}

#[derive(Deserialize, Clone, Debug)]
pub struct AssetPattern {
    policy: Option<Vec<u8>>,
    name: Option<AsciiPattern>,
    coin: Option<CoinPattern>,
}

impl PatternOf<Multiasset> for AssetPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let a = self
            .policy
            .as_ref()
            .map(|x| MatchOutcome::if_equal(x.as_slice(), subject.policy_id.as_ref()))
            .unwrap_or(MatchOutcome::Positive);

        let c = self
            .coin
            .is_any_match(subject.assets.iter().map(|a| &a.output_coin));

        MatchOutcome::fold_all_of([a, c].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct OutputPattern {
    address: AddressPattern,
    lovelace: Option<CoinPattern>,
    any_asset: Option<AssetPattern>,
}

impl PatternOf<TxOutput> for OutputPattern {
    fn is_match(&self, subject: &TxOutput) -> MatchOutcome {
        let a = self.address.is_match(subject.address.as_ref());

        let b = self.lovelace.is_match(&subject.coin);

        let c = self.any_asset.is_any_match(subject.assets.iter());

        MatchOutcome::fold_all_of([a, b, c].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct TxPattern {
    // TODO: containing_block: BlockPattern
    any_output: Option<OutputPattern>,
    any_address: Option<AddressPattern>,
    any_asset: Option<AssetPattern>,
}

impl PatternOf<ParsedTx> for TxPattern {
    fn is_match(&self, tx: &ParsedTx) -> MatchOutcome {
        let a = self.any_output.is_any_match(tx.outputs.iter());

        // let b

        let c = self
            .any_asset
            .is_any_match(tx.outputs.iter().flat_map(|o| o.assets.iter()));

        MatchOutcome::fold_all_of([a, c].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum Predicate {
    Match(TxPattern),
    Not(Box<Predicate>),
    AnyOf(Vec<Predicate>),
    AllOf(Vec<Predicate>),
}

fn eval_tx(tx: &ParsedTx, predicate: &Predicate) -> MatchOutcome {
    match predicate {
        Predicate::Not(x) => !eval_tx(tx, predicate),
        Predicate::AnyOf(x) => {
            let o = x.iter().map(|x| eval_tx(tx, x));
            MatchOutcome::fold_any_of(o)
        }
        Predicate::AllOf(x) => {
            let o = x.iter().map(|x| eval_tx(tx, x));
            MatchOutcome::fold_all_of(o)
        }
        Predicate::Match(x) => x.is_match(tx),
    }
}

fn eval_block(block: &ParsedBlock, predicate: &Predicate) -> MatchOutcome {
    let outcomes = block
        .body
        .iter()
        .flat_map(|b| b.tx.iter())
        .map(|tx| eval_tx(tx, predicate));

    MatchOutcome::fold_any_of(outcomes)
}

pub fn eval(record: &Record, predicate: &Predicate) -> MatchOutcome {
    match record {
        Record::ParsedTx(x) => eval_tx(x, predicate),
        Record::ParsedBlock(x) => eval_block(x, predicate),
        _ => MatchOutcome::Uncertain,
    }
}
