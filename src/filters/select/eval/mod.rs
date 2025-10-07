use std::{ops::Deref, str::FromStr};

use pallas::interop::utxorpc::spec::cardano::{
    Asset, AuxData, Metadata, Metadatum, Multiasset, TxInput, TxOutput,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::framework::*;

mod address;
mod assets;
mod bytes;
mod cip14;
mod metadata;
mod serde_ext;

#[cfg(test)]
mod testing;

pub use address::*;
pub use assets::*;
pub use bytes::*;
pub use cip14::*;
pub use metadata::*;
pub use serde_ext::*;

#[derive(Clone, Copy, Debug, PartialEq)]
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

            folded += item;
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

    fn is_some_match(&self, subject: Option<S>) -> MatchOutcome {
        self.is_any_match(subject.into_iter())
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

impl PatternOf<&[u8]> for FlexBytes {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        MatchOutcome::if_equal(self.deref(), subject)
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

impl PatternOf<&[u8]> for TextPattern {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        let subject = match String::from_utf8(subject.to_vec()) {
            Ok(subject) => subject,
            Err(_) => return MatchOutcome::Uncertain,
        };

        self.is_match(subject.as_str())
    }
}

impl PatternOf<&Metadatum> for TextPattern {
    fn is_match(&self, subject: &Metadatum) -> MatchOutcome {
        match subject.metadatum.as_ref() {
            Some(pallas::interop::utxorpc::spec::cardano::metadatum::Metadatum::Text(subject)) => {
                self.is_match(subject.as_str())
            }
            _ => MatchOutcome::Negative,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DatumPattern {
    hash: Option<Vec<u8>>,
}

impl FromBech32 for DatumPattern {
    fn from_bech32_parts(hrp: &str, content: Vec<u8>) -> Option<Self> {
        match hrp {
            "datum" => Some(Self {
                hash: Some(content),
            }),
            _ => None,
        }
    }
}

impl FromStr for DatumPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl PatternOf<&[u8]> for DatumPattern {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        self.hash.is_match(subject)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScriptPattern {
    hash: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct OutputPattern {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    address: Option<StringOrStruct<AddressPattern>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    lovelace: Option<CoinPattern>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assets: Vec<StringOrStruct<AssetPattern>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    datum: Option<StringOrStruct<DatumPattern>>,
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

        let d = self
            .datum
            .is_some_match(subject.datum.as_ref().map(|d| d.hash.as_ref()));

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InputPattern {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    address: Option<StringOrStruct<AddressPattern>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assets: Vec<StringOrStruct<AssetPattern>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    lovelace: Option<CoinPattern>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    datum: Option<StringOrStruct<DatumPattern>>,
    // u5c redeemer structure is not suitable, is lacks a datum hash (and it also contains a
    // redundant purpose flag) redeemer: Option<DatumPattern>,
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

        let d = self
            .datum
            .is_some_match(as_output.datum.as_ref().map(|x| x.hash.as_ref()));

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MintPattern {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assets: Vec<StringOrStruct<AssetPattern>>,
    // the u5c struct is not suitable, it lacks the redeemer value
    // redeemer: Option<DatumPattern>,
}

impl PatternOf<&Multiasset> for MintPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let a = self.assets.iter().map(|x| x.is_match(subject));

        let a = MatchOutcome::fold_all_of(a);

        MatchOutcome::fold_all_of([a].into_iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct TxPattern {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    inputs: Vec<InputPattern>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    outputs: Vec<OutputPattern>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    mint: Vec<MintPattern>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    metadata: Vec<MetadataPattern>,
    // the u5c struct is not suitable, it lacks hash for the scripts
    // scripts: Vec<ScriptPattern>,
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

        let d = self
            .metadata
            .iter()
            .map(|x| x.is_any_match(tx.auxiliary.iter()));

        let d = MatchOutcome::fold_all_of(d);

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

pub type SlotPattern = NumericPattern<u64>;

pub type EraPattern = NumericPattern<u8>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlockPattern {
    hash: Option<Vec<u8>>,
    slot: Option<SlotPattern>,
    era: Option<EraPattern>,
    txs: Vec<TxPattern>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Pattern {
    Block(BlockPattern),
    Tx(TxPattern),
    Address(StringOrStruct<AddressPattern>),
    Asset(StringOrStruct<AssetPattern>),
    Input(InputPattern),
    Output(OutputPattern),
    Mint(MintPattern),
    Metadata(StringOrStruct<MetadataPattern>),
    Datum(StringOrStruct<DatumPattern>),
}

impl From<AssetPattern> for Pattern {
    fn from(value: AssetPattern) -> Self {
        Pattern::Asset(StringOrStruct(value))
    }
}

impl From<AddressPattern> for Pattern {
    fn from(value: AddressPattern) -> Self {
        Pattern::Address(StringOrStruct(value))
    }
}

impl From<DatumPattern> for Pattern {
    fn from(value: DatumPattern) -> Self {
        Pattern::Datum(StringOrStruct(value))
    }
}

impl From<MetadataPattern> for Pattern {
    fn from(value: MetadataPattern) -> Self {
        Pattern::Metadata(StringOrStruct(value))
    }
}

impl FromBech32 for Pattern {
    fn from_bech32_parts(hrp: &str, content: Vec<u8>) -> Option<Self> {
        match hrp {
            "asset" => AssetPattern::from_bech32_parts(hrp, content).map(From::from),
            "addr" => AddressPattern::from_bech32_parts(hrp, content).map(From::from),
            "addr_test" => AddressPattern::from_bech32_parts(hrp, content).map(From::from),
            "stake" => AddressPattern::from_bech32_parts(hrp, content).map(From::from),
            "datum" => DatumPattern::from_bech32_parts(hrp, content).map(From::from),
            _ => None,
        }
    }
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(p) = Self::from_bech32(s) {
            return Ok(p);
        }

        if let Ok(p) = MetadataPattern::from_str(s) {
            return Ok(p.into());
        }

        anyhow::bail!("can't parse pattern from string");
    }
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

fn iter_tx_assets(tx: &ParsedTx) -> impl Iterator<Item = &Multiasset> {
    let a = tx.outputs.iter().flat_map(|x| x.assets.iter());

    let b = tx.mint.iter();

    a.chain(b)
}

fn iter_tx_datums(tx: &ParsedTx) -> impl Iterator<Item = &[u8]> {
    let a = tx
        .outputs
        .iter()
        .flat_map(|x| &x.datum)
        .map(|x| x.hash.as_ref());

    a
}

impl PatternOf<&ParsedTx> for Pattern {
    fn is_match(&self, subject: &ParsedTx) -> MatchOutcome {
        match self {
            Pattern::Block(_) => MatchOutcome::Negative,
            Pattern::Tx(x) => x.is_match(subject),
            Pattern::Address(x) => x.is_any_match(iter_tx_addresses(subject)),
            Pattern::Asset(x) => x.is_any_match(iter_tx_assets(subject)),
            Pattern::Input(x) => x.is_any_match(subject.inputs.iter()),
            Pattern::Output(x) => x.is_any_match(subject.outputs.iter()),
            Pattern::Mint(x) => x.is_any_match(subject.mint.iter()),
            Pattern::Metadata(x) => x.is_any_match(subject.auxiliary.iter()),
            Pattern::Datum(x) => x.is_any_match(iter_tx_datums(subject)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Predicate {
    #[serde(rename = "match")]
    Match(StringOrStruct<Pattern>),

    #[serde(rename = "not")]
    Not(Box<StringOrStruct<Predicate>>),

    #[serde(rename = "any")]
    AnyOf(Vec<StringOrStruct<Predicate>>),

    #[serde(rename = "all")]
    AllOf(Vec<StringOrStruct<Predicate>>),
}

impl Predicate {
    pub fn any_of(p: Vec<Self>) -> Self {
        Predicate::AnyOf(p.into_iter().map(StringOrStruct).collect())
    }

    pub fn all_of(p: Vec<Self>) -> Self {
        Predicate::AllOf(p.into_iter().map(StringOrStruct).collect())
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(p: Self) -> Self {
        Predicate::Not(Box::new(StringOrStruct(p)))
    }
}

impl From<Pattern> for Predicate {
    fn from(value: Pattern) -> Self {
        Predicate::Match(StringOrStruct(value))
    }
}

impl FromStr for Predicate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = Pattern::from_str(s)?;
        Ok(inner.into())
    }
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
        Record::Cardano(record) => match record {
            cardano::Record::ParsedTx(x) => eval_tx(x, predicate),
            cardano::Record::ParsedBlock(x) => eval_block(x, predicate),
            _ => {
                warn!("The select filter is valid only for ParsedTx / ParsedBlock records");
                MatchOutcome::Uncertain
            }
        },
        _ => {
            warn!("The select filter is valid only for Cardano ParsedTx / ParsedBlock records");
            MatchOutcome::Uncertain
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn empty_tx_pattern() {
        let pattern = Pattern::Tx(TxPattern::default());

        let positives = testing::find_positive_test_vectors(pattern);
        assert_eq!(positives, vec![0, 1, 2, 3]);
    }

    #[test]
    fn parse_pattern() {
        let pattern = Pattern::from_str("addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x").unwrap();
        assert!(matches!(pattern, Pattern::Address(..)));

        let pattern = Pattern::from_str("asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92").unwrap();
        assert!(matches!(pattern, Pattern::Asset(..)));

        let pattern = Pattern::from_str("datum1kthqfw4769ejpkx3h8le45yxaph5fmzdnur2s4").unwrap();
        assert!(matches!(pattern, Pattern::Datum(..)));

        let pattern = Pattern::from_str("#8888").unwrap();
        assert!(matches!(pattern, Pattern::Metadata(..)));
    }

    #[test]
    fn deser_predicate() {
        serde_json::from_str::<StringOrStruct<Predicate>>("\"addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x\"").unwrap();

        serde_json::from_str::<StringOrStruct<Predicate>>("\"#127\"").unwrap();

        serde_json::from_str::<StringOrStruct<Predicate>>(
            r#"{
                "all": [
                    "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
                    { "match": "asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92" },
                    { "not": "datum1kthqfw4769ejpkx3h8le45yxaph5fmzdnur2s4" },
                    {
                        "all": [{
                            "match": {
                                "mint": {
                                    "assets": ["asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92"]
                                }
                            }
                        }]
                    }
                ]
            }"#,
        ).unwrap();

        serde_json::from_str::<StringOrStruct<Predicate>>(
            r#"{
                "match": {
                    "tx": {
                        "inputs": [{
                            "address": "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
                            "datum": "datum1kthqfw4769ejpkx3h8le45yxaph5fmzdnur2s4"
                        }],
                        "outputs": [{
                            "address": "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x",
                            "lovelace": { "between": [1000000, 5000000 ] },
                            "datum": "datum1kthqfw4769ejpkx3h8le45yxaph5fmzdnur2s4"
                        }],
                        "mint": [{
                            "assets": ["asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92"]
                        }],
                        "metadata": [{
                            "label": 127
                        }]
                    }
                }
            }"#,
        ).unwrap();
    }
}
