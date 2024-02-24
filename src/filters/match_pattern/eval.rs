use serde::Deserialize;
use utxorpc::proto::cardano::v1::{
    Asset, AuxData, Metadata, Metadatum, Multiasset, TxInput, TxOutput,
};

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

pub trait PatternOf<S> {
    fn is_match(&self, subject: S) -> MatchOutcome;

    fn is_any_match<'a, I>(&self, iter: I) -> MatchOutcome
    where
        I: Iterator<Item = S>,
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
    fn is_match(&self, subject: S) -> MatchOutcome {
        match self {
            Some(x) => x.is_match(subject),
            // the absence of a pattern matches everything
            None => MatchOutcome::Positive,
        }
    }
}

impl PatternOf<&[u8]> for Vec<u8> {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        MatchOutcome::if_equal(self.as_ref(), subject)
    }
}

impl PatternOf<bool> for bool {
    fn is_match(&self, subject: bool) -> MatchOutcome {
        MatchOutcome::if_equal(self, &subject)
    }
}

impl PatternOf<u64> for u64 {
    fn is_match(&self, subject: u64) -> MatchOutcome {
        MatchOutcome::if_equal(self, &subject)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum NumericPattern<I: Ord + Eq> {
    Exact(I),
    Gte(I),
    Lte(I),
    Between(I, I),
}

pub type CoinPattern = NumericPattern<u64>;

impl PatternOf<u64> for CoinPattern {
    fn is_match(&self, subject: u64) -> MatchOutcome {
        match self {
            CoinPattern::Exact(x) => MatchOutcome::if_true(subject == *x),
            CoinPattern::Gte(x) => MatchOutcome::if_true(subject >= *x),
            CoinPattern::Lte(x) => MatchOutcome::if_true(subject <= *x),
            CoinPattern::Between(a, b) => MatchOutcome::if_true(subject >= *a && subject <= *b),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum TextPattern {
    Exact(String),
    // TODO: Regex
}

impl PatternOf<&str> for TextPattern {
    fn is_match(&self, subject: &str) -> MatchOutcome {
        match self {
            TextPattern::Exact(x) => MatchOutcome::if_equal(x.as_str(), subject),
        }
    }
}

impl PatternOf<&Metadatum> for TextPattern {
    fn is_match(&self, subject: &Metadatum) -> MatchOutcome {
        match subject.metadatum.as_ref() {
            Some(utxorpc::proto::cardano::v1::metadatum::Metadatum::Text(subject)) => {
                self.is_match(subject.as_str())
            }
            _ => MatchOutcome::Negative,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AssetPattern {
    name: Option<Vec<u8>>,
    ascii_name: Option<TextPattern>,
    coin: Option<CoinPattern>,
}

impl PatternOf<&Asset> for AssetPattern {
    fn is_match(&self, subject: &Asset) -> MatchOutcome {
        let a = self.name.is_match(subject.name.as_ref());

        let b = todo!();

        let c = todo!();

        MatchOutcome::fold_all_of([a, b, c].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatumPattern {
    hash: Option<Vec<u8>>,
}

impl PatternOf<&[u8]> for DatumPattern {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        self.hash.is_match(subject)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScriptPattern {
    hash: Option<Vec<u8>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MultiAssetPattern {
    policy: Option<Vec<u8>>,
    assets: Vec<AssetPattern>,
}

impl PatternOf<&Multiasset> for MultiAssetPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let a = self
            .policy
            .as_ref()
            .map(|x| MatchOutcome::if_equal(x.as_slice(), &subject.policy_id))
            .unwrap_or(MatchOutcome::Positive);

        let b = self
            .assets
            .iter()
            .map(|x| x.is_any_match(subject.assets.iter()));

        let b = MatchOutcome::fold_all_of(b);

        MatchOutcome::fold_all_of([a, b].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct OutputPattern {
    address: Option<AddressPattern>,
    lovelace: Option<CoinPattern>,
    assets: Vec<MultiAssetPattern>,
    datum: Option<DatumPattern>,
}

impl PatternOf<&TxOutput> for OutputPattern {
    fn is_match(&self, subject: &TxOutput) -> MatchOutcome {
        let a = self.address.is_match(subject.address.as_ref());

        let b = self.lovelace.is_match(subject.coin);

        let c = self
            .assets
            .iter()
            .map(|x| x.is_any_match(subject.assets.iter()));

        let c = MatchOutcome::fold_all_of(c);

        let d = self.datum.is_match(subject.datum_hash.as_ref());

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct InputPattern {
    address: Option<AddressPattern>,
    assets: Vec<MultiAssetPattern>,
    lovelace: Option<CoinPattern>,
    datum: Option<DatumPattern>,
    redeemer: Option<DatumPattern>,
}

impl PatternOf<&TxInput> for InputPattern {
    fn is_match(&self, subject: &TxInput) -> MatchOutcome {
        let as_output = match subject.as_output.as_ref() {
            Some(x) => x,
            None => return MatchOutcome::Uncertain,
        };

        let a = self.address.is_match(as_output.address.as_ref());

        let b = self.lovelace.is_match(as_output.coin);

        let c = self
            .assets
            .iter()
            .map(|x| x.is_any_match(as_output.assets.iter()));

        let c = MatchOutcome::fold_all_of(c);

        let d = self.datum.is_match(as_output.datum_hash.as_ref());

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MintPattern {
    assets: Vec<MultiAssetPattern>,
    redeemer: Option<DatumPattern>,
}

impl PatternOf<&Multiasset> for MintPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let a = self.assets.iter().map(|x| x.is_match(subject));

        let a = MatchOutcome::fold_all_of(a);

        let b = todo!();

        MatchOutcome::fold_all_of([a, b].into_iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum MetadatumPattern {
    Text(TextPattern),
    Int(NumericPattern<i64>),
}

impl PatternOf<&Metadatum> for MetadatumPattern {
    fn is_match(&self, subject: &Metadatum) -> MatchOutcome {
        match self {
            MetadatumPattern::Text(x) => x.is_match(subject),
            MetadatumPattern::Int(_) => todo!(),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MetadataPattern {
    label: Option<u64>,
    value: Option<MetadatumPattern>,
    nested_value: Option<MetadatumPattern>,
}

impl PatternOf<&Metadata> for MetadataPattern {
    fn is_match(&self, subject: &Metadata) -> MatchOutcome {
        let a = self.label.is_match(subject.label);

        let b = self.value.is_any_match(subject.value.iter());

        let c = todo!();

        MatchOutcome::fold_all_of([a, b, c].into_iter())
    }
}

impl PatternOf<&AuxData> for MetadataPattern {
    fn is_match(&self, subject: &AuxData) -> MatchOutcome {
        self.is_any_match(subject.metadata.iter())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct TxPattern {
    inputs: Vec<InputPattern>,
    outputs: Vec<OutputPattern>,
    mint: Vec<MintPattern>,
    metadata: Vec<MetadataPattern>,
    scripts: Vec<ScriptPattern>,
}

impl PatternOf<&ParsedTx> for TxPattern {
    fn is_match(&self, tx: &ParsedTx) -> MatchOutcome {
        let a = self.inputs.iter().map(|x| x.is_any_match(tx.inputs.iter()));

        let a = MatchOutcome::fold_all_of(a);

        let b = self
            .outputs
            .iter()
            .map(|x| x.is_any_match(tx.outputs.iter()));

        let b = MatchOutcome::fold_all_of(b);

        let c = self.mint.iter().map(|x| x.is_any_match(tx.mint.iter()));

        let c = MatchOutcome::fold_all_of(c);

        let d = todo!();

        let e = todo!();

        MatchOutcome::fold_all_of([a, b, c, d, e].into_iter())
    }
}

pub type SlotPattern = NumericPattern<u64>;

pub type EraPattern = NumericPattern<u8>;

#[derive(Deserialize, Clone, Debug)]
pub struct BlockPattern {
    hash: Option<Vec<u8>>,
    slot: Option<SlotPattern>,
    era: Option<EraPattern>,
    txs: Vec<TxPattern>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum Pattern {
    Block(BlockPattern),
    Tx(TxPattern),
    Address(AddressPattern),
    Input(InputPattern),
    Output(OutputPattern),
    Mint(MintPattern),
    Metadata(MetadataPattern),
}

fn iter_tx_addresses(tx: &ParsedTx) -> impl Iterator<Item = &[u8]> {
    let a = tx.outputs.iter().map(|x| x.address.as_ref());

    let b = tx
        .inputs
        .iter()
        .flat_map(|x| x.as_output.as_ref())
        .map(|x| x.address.as_ref());

    a.chain(b)
}

impl PatternOf<&ParsedTx> for Pattern {
    fn is_match(&self, subject: &ParsedTx) -> MatchOutcome {
        match self {
            Pattern::Block(_) => MatchOutcome::Negative,
            Pattern::Tx(x) => x.is_match(subject),
            Pattern::Address(x) => x.is_any_match(iter_tx_addresses(subject)),
            Pattern::Input(x) => x.is_any_match(subject.inputs.iter()),
            Pattern::Output(x) => x.is_any_match(subject.outputs.iter()),
            Pattern::Mint(x) => x.is_any_match(subject.mint.iter()),
            Pattern::Metadata(x) => x.is_any_match(subject.auxiliary.iter()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum Predicate {
    Match(Pattern),
    Not(Box<Predicate>),
    AnyOf(Vec<Predicate>),
    AllOf(Vec<Predicate>),
}

fn eval_tx(tx: &ParsedTx, predicate: &Predicate) -> MatchOutcome {
    match predicate {
        Predicate::Not(x) => !eval_tx(tx, x),
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
