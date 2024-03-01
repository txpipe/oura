use std::ops::Deref;

use serde::{Deserialize, Serialize};
use tracing::warn;
use utxorpc::proto::cardano::v1::{
    Asset, AuxData, Metadata, Metadatum, Multiasset, Redeemer, TxInput, TxOutput,
};

use crate::framework::*;

use super::{address::AddressPattern, FlexBytes};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
            Some(utxorpc::proto::cardano::v1::metadatum::Metadatum::Text(subject)) => {
                self.is_match(subject.as_str())
            }
            _ => MatchOutcome::Negative,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AssetPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    policy: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ascii_name: Option<TextPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
    coin: Option<CoinPattern>,
}

impl PatternOf<(&[u8], &Asset)> for AssetPattern {
    fn is_match(&self, subject: (&[u8], &Asset)) -> MatchOutcome {
        let (subject_policy, subject_asset) = subject;

        let a = self.policy.is_match(subject_policy);

        let b = self.name.is_match(subject_asset.name.as_ref());

        let c = self.ascii_name.is_match(subject_asset.name.as_ref());

        let d = self.coin.is_match(subject_asset.output_coin);

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

impl PatternOf<&Multiasset> for AssetPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let policy = subject.policy_id.as_ref();

        let subjects = subject.assets.iter().map(|x| (policy, x));

        self.is_any_match(subjects)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatumPattern {
    hash: Option<Vec<u8>>,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OutputPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<AddressPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
    lovelace: Option<CoinPattern>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    assets: Vec<AssetPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<AddressPattern>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    assets: Vec<AssetPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
    lovelace: Option<CoinPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
    datum: Option<DatumPattern>,
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

        let d = self.datum.is_match(as_output.datum_hash.as_ref());

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MintPattern {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assets: Vec<AssetPattern>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MetadatumPattern {
    Text(TextPattern),
    Int(NumericPattern<i64>),
    // TODO: bytes, array, map
}

impl PatternOf<&Metadatum> for MetadatumPattern {
    fn is_match(&self, subject: &Metadatum) -> MatchOutcome {
        match self {
            MetadatumPattern::Text(x) => x.is_match(subject),
            MetadatumPattern::Int(_) => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetadataPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<MetadatumPattern>,
}

impl PatternOf<&Metadata> for MetadataPattern {
    fn is_match(&self, subject: &Metadata) -> MatchOutcome {
        let a = self.label.is_match(subject.label);

        let b = self.value.is_any_match(subject.value.iter());

        MatchOutcome::fold_all_of([a, b].into_iter())
    }
}

impl PatternOf<&AuxData> for MetadataPattern {
    fn is_match(&self, subject: &AuxData) -> MatchOutcome {
        self.is_any_match(subject.metadata.iter())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockPattern {
    hash: Option<Vec<u8>>,
    slot: Option<SlotPattern>,
    era: Option<EraPattern>,
    txs: Vec<TxPattern>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Predicate {
    #[serde(rename = "match")]
    Match(Pattern),

    #[serde(rename = "not")]
    Not(Box<Predicate>),

    #[serde(rename = "any")]
    AnyOf(Vec<Predicate>),

    #[serde(rename = "all")]
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
        _ => {
            warn!("The select filter is valid only with ParsedTx & ParsedBlock records");
            MatchOutcome::Uncertain
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utxorpc::proto::cardano::v1::Tx;

    fn multiasset_combo(policy_hex: &str, asset_prefix: &str) -> Multiasset {
        Multiasset {
            policy_id: hex::decode(policy_hex).unwrap().into(),
            assets: vec![
                Asset {
                    name: format!("{asset_prefix}1").as_bytes().to_vec().into(),
                    output_coin: 345000000,
                    mint_coin: 0,
                },
                Asset {
                    name: format!("{asset_prefix}2").as_bytes().to_vec().into(),
                    output_coin: 345000000,
                    mint_coin: 0,
                },
            ],
        }
    }

    fn test_vectors() -> Vec<Tx> {
        let _0 = Tx::default();

        let _1 = Tx {
            outputs: vec![TxOutput {
                address: hex::decode("019493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251").unwrap().into(),
                coin: 123000000,
                assets: vec![
                    multiasset_combo("019493315cd92eb5d8c4304e67b7e16ae36d61de", "abc"),
                    multiasset_combo("b2ee04babed17320d8d1b9ff9ad086e86f44ec4d", "123")
                ],
                datum_hash: hex::decode("923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec").unwrap().into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let _2 = Tx {
            outputs: vec![TxOutput {
                address: hex::decode("019493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251").unwrap().into(),
                coin: 123000000,
                assets: vec![
                    multiasset_combo("019493315cd92eb5d8c4304e67b7e16ae36d61de", "abc"),
                ],
                datum_hash: hex::decode("923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec").unwrap().into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let _3 = Tx {
            outputs: vec![TxOutput {
                address: hex::decode("019493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251").unwrap().into(),
                coin: 123000000,
                assets: vec![
                    multiasset_combo("b2ee04babed17320d8d1b9ff9ad086e86f44ec4d", "123")
                ],
                datum_hash: hex::decode("923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec").unwrap().into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        vec![_0, _1, _2, _3]
    }

    fn find_positive_test_vectors(predicate: Predicate) -> Vec<usize> {
        let subjects = test_vectors();

        subjects
            .into_iter()
            .enumerate()
            .filter_map(|(idx, subject)| match eval_tx(&subject, &predicate) {
                MatchOutcome::Positive => Some(idx),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn empty_tx_pattern() {
        let pattern = Pattern::Tx(TxPattern::default());

        let positives = find_positive_test_vectors(Predicate::Match(pattern));
        assert_eq!(positives, vec![0]);
    }

    #[test]
    fn output_multiasset_asset_name_match() {
        let pattern = |token: &str| {
            Pattern::Output(OutputPattern {
                assets: vec![AssetPattern {
                    name: Some(token.into()),
                    ..Default::default()
                }],
                ..Default::default()
            })
        };

        let positives = find_positive_test_vectors(Predicate::Match(pattern("abc1")));
        assert_eq!(positives, vec![1, 2]);

        let positives = find_positive_test_vectors(Predicate::Match(pattern("1231")));
        assert_eq!(positives, vec![1, 3]);

        let positives = find_positive_test_vectors(Predicate::Match(pattern("doesntexist")));
        assert_eq!(positives, Vec::<usize>::new());
    }

    #[test]
    fn serde() {
        let predicate1 = Predicate::Match(Pattern::Output(OutputPattern {
            assets: vec![AssetPattern {
                policy: Some("b2ee04babed17320d8d1b9ff9ad086e86f44ec4d".into()),
                name: Some("abc1".into()),
                ..Default::default()
            }],
            ..Default::default()
        }));

        let predicate2 = Predicate::Match(Pattern::Address(AddressPattern {
            payment_part: Some("b2ee04babed17320d8d1b9ff9ad086e86f44ec4d".into()),
            ..Default::default()
        }));

        let predicate = Predicate::AnyOf(vec![predicate1, predicate2]);

        println!("{}", serde_json::to_string_pretty(&predicate).unwrap());
    }
}