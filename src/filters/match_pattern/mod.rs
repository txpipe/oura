use gasket::framework::*;
use pallas::{
    crypto::hash::Hash,
    ledger::addresses::{
        Address, PaymentKeyHash, ShelleyDelegationPart, ShelleyPaymentPart, StakeAddress,
        StakeKeyHash,
    },
    network::miniprotocols::Point,
};
use serde::Deserialize;
use tracing::error;

mod eval;

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
    key: Option<TextPattern>,
    value: Option<TextPattern>,
}

#[derive(Clone, Debug)]
#[serde(rename_all = "snake_case")]
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
    SomeWithdrawalMatches(OutputPattern),
    SomeAddressMatches(AddressPattern),
    Not(Box<TxPredicate>),
    AnyOf(Vec<TxPredicate>),
    AllOf(Vec<TxPredicate>),
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
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
