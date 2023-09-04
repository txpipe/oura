use std::str::FromStr;

use gasket::framework::*;
use pallas::{
    crypto::hash::Hash,
    ledger::addresses::{Address, ShelleyDelegationPart, ShelleyPaymentPart},
    network::miniprotocols::Point,
};
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use tracing::{error, warn};

mod eval;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "filter-match-pattern", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    predicate: TxPredicate,

    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    pass_count: gasket::metrics::Counter,

    #[metric]
    drop_count: gasket::metrics::Counter,

    #[metric]
    inconclusive_count: gasket::metrics::Counter,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Worker {}
    }
}

fn eval_record(stage: &Stage, point: &Point, record: &Record) -> Option<Record> {
    match eval::eval(&stage.predicate, point, record) {
        Ok(pass) => {
            if pass {
                stage.pass_count.inc(1);
                Some(record.clone())
            } else {
                stage.drop_count.inc(1);
                None
            }
        }
        Err(eval::Error::Inconclusive(msg)) => {
            warn!(msg);
            stage.inconclusive_count.inc(1);
            None
        }
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    stage.ops_count.inc(1);

    if let Some(record) = unit.record() {
        eval_record(stage, unit.point(), record)
            .map(|x| unit.new_record(x))
            .map(|x| vec![x])
            .unwrap_or(vec![])
    } else {
        vec![unit.clone()]
    }
});

#[derive(Clone, Debug)]
pub struct AsciiPattern {}

#[derive(Clone, Debug)]
pub enum QuantityPattern {
    Equals(u64),
    RangeInclusive(u64, u64),
    Greater(u64),
    GreaterOrEqual(u64),
    Lower(u64),
    LowerOrEqual(u64),
}

#[derive(Clone, Debug)]
pub struct BlockPattern {
    pub slot_before: Option<u64>,
    pub slot_after: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct UtxoRefPattern {
    tx_hash: Option<Hash<32>>,
    output_idx: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct DatumPattern {
    hash: Option<Hash<32>>,
}

#[derive(Clone, Debug)]
pub struct WithdrawalPattern {
    quantity: Option<QuantityPattern>,
    // reward account pattern?
}

#[derive(Clone, Debug)]
pub struct AssetPattern {
    policy: Option<Vec<u8>>,
    name: Option<AsciiPattern>,
    quantity: Option<QuantityPattern>,
}

#[derive(Clone, Debug)]
pub enum AddressPattern {
    Exact(Address),
    Payment(ShelleyPaymentPart),
    Delegation(ShelleyDelegationPart),
}

#[derive(Clone, Debug)]
pub struct InputPattern {
    assets: Option<AssetPattern>,
    from: Option<AddressPattern>,
    utxo: Option<UtxoRefPattern>,
    datum: Option<DatumPattern>,
}

#[derive(Clone, Debug)]
pub struct OutputPattern {
    assets: Option<AssetPattern>,
    to: Option<AddressPattern>,
    datum: Option<DatumPattern>,
}

#[derive(Clone, Debug)]
pub struct MetadataPattern {
    label: Option<u32>,
    key: Option<AsciiPattern>,
    value: Option<AsciiPattern>,
}

#[derive(Clone, Debug, DeserializeFromStr)]
pub enum TxPredicate {
    HashEquals(Option<Hash<32>>),
    IsValid(Option<bool>),
    BlockMatches(BlockPattern),
    SomeInputMatches(InputPattern),
    TotalInputAssetsMatch(AssetPattern),
    SomeInputAddressMatches(AddressPattern),
    SomeInputAssetMatches(AssetPattern),
    SomeInputDatumMatches(DatumPattern),
    TotalOutputAssetsMatch(AssetPattern),
    SomeOutputMatches(OutputPattern),
    SomeOutputAddressMatches(AddressPattern),
    SomeOutputDatumMatches(DatumPattern),
    SomeOutputAssetMatches(AssetPattern),
    SomeMintedAssetMatches(AssetPattern),
    SomeBurnedAssetMatches(AssetPattern),
    SomeMetadataMatches(MetadataPattern),
    SomeCollateralMatches(InputPattern),
    CollateralReturnMatches(OutputPattern),
    TotalCollateralMatches(QuantityPattern),
    SomeWithdrawalMatches(WithdrawalPattern),
    SomeAddressMatches(AddressPattern),
    Not(Box<TxPredicate>),
    AnyOf(Vec<TxPredicate>),
    AllOf(Vec<TxPredicate>),
}

impl FromStr for TxPredicate {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub predicate: TxPredicate,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            predicate: self.predicate,
            ops_count: Default::default(),
            pass_count: Default::default(),
            drop_count: Default::default(),
            inconclusive_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
