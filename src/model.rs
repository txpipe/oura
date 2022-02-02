use std::fmt::Display;

use merge::Merge;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MetadatumRendition {
    MapJson(JsonValue),
    ArrayJson(JsonValue),
    IntScalar(i64),
    TextScalar(String),
    BytesHex(String),
}

impl Display for MetadatumRendition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadatumRendition::MapJson(x) => x.fmt(f),
            MetadatumRendition::ArrayJson(x) => x.fmt(f),
            MetadatumRendition::IntScalar(x) => x.fmt(f),
            MetadatumRendition::TextScalar(x) => x.fmt(f),
            MetadatumRendition::BytesHex(x) => x.fmt(f),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetadataRecord {
    pub label: String,

    #[serde(flatten)]
    pub content: MetadatumRendition,
}

impl From<MetadataRecord> for EventData {
    fn from(x: MetadataRecord) -> Self {
        EventData::Metadata(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CIP25AssetRecord {
    pub version: String,
    pub policy: String,
    pub asset: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub media_type: Option<String>,
    pub description: Option<String>,
    pub raw_json: JsonValue,
}

impl From<CIP25AssetRecord> for EventData {
    fn from(x: CIP25AssetRecord) -> Self {
        EventData::CIP25Asset(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInputRecord {
    pub tx_id: String,
    pub index: u64,
}

impl From<TxInputRecord> for EventData {
    fn from(x: TxInputRecord) -> Self {
        EventData::TxInput(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputAssetRecord {
    pub policy: String,
    pub asset: String,
    pub amount: u64,
}

impl From<OutputAssetRecord> for EventData {
    fn from(x: OutputAssetRecord) -> Self {
        EventData::OutputAsset(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutputRecord {
    pub address: String,
    pub amount: u64,
    pub assets: Option<Vec<OutputAssetRecord>>,
}

impl From<TxOutputRecord> for EventData {
    fn from(x: TxOutputRecord) -> Self {
        EventData::TxOutput(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintRecord {
    pub policy: String,
    pub asset: String,
    pub quantity: i64,
}

impl From<MintRecord> for EventData {
    fn from(x: MintRecord) -> Self {
        EventData::Mint(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransactionRecord {
    pub hash: String,
    pub fee: u64,
    pub ttl: Option<u64>,
    pub validity_interval_start: Option<u64>,
    pub network_id: Option<u32>,
    pub input_count: usize,
    pub output_count: usize,
    pub mint_count: usize,
    pub total_output: u64,

    // include_details
    pub metadata: Option<Vec<MetadataRecord>>,
    pub inputs: Option<Vec<TxInputRecord>>,
    pub outputs: Option<Vec<TxOutputRecord>>,
    pub mint: Option<Vec<MintRecord>>,
}

impl From<TransactionRecord> for EventData {
    fn from(x: TransactionRecord) -> Self {
        EventData::Transaction(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Merge, Default)]
pub struct EventContext {
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub timestamp: Option<u64>,
    pub tx_idx: Option<usize>,
    pub tx_hash: Option<String>,
    pub input_idx: Option<usize>,
    pub output_idx: Option<usize>,
    pub output_address: Option<String>,
    pub certificate_idx: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StakeCredential {
    AddrKeyhash(String),
    Scripthash(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockRecord {
    pub body_size: usize,
    pub issuer_vkey: String,
    pub tx_count: usize,
    pub slot: u64,
    pub hash: String,
    pub number: u64,
    pub previous_hash: String,
}

#[derive(Serialize, Deserialize, Display, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventData {
    Block(BlockRecord),
    BlockEnd(BlockRecord),
    Transaction(TransactionRecord),
    TransactionEnd(TransactionRecord),
    TxInput(TxInputRecord),
    TxOutput(TxOutputRecord),
    OutputAsset(OutputAssetRecord),
    Metadata(MetadataRecord),

    #[serde(rename = "cip25_asset")]
    CIP25Asset(CIP25AssetRecord),

    Mint(MintRecord),
    Collateral {
        tx_id: String,
        index: u64,
    },
    NativeScript {},
    PlutusScript {
        data: String,
    },
    StakeRegistration {
        credential: StakeCredential,
    },
    StakeDeregistration {
        credential: StakeCredential,
    },
    StakeDelegation {
        credential: StakeCredential,
        pool_hash: String,
    },
    PoolRegistration {
        operator: String,
        vrf_keyhash: String,
        pledge: u64,
        cost: u64,
        margin: f64,
        reward_account: String,
        pool_owners: Vec<String>,
        relays: Vec<String>,
        pool_metadata: Option<String>,
    },
    PoolRetirement {
        pool: String,
        epoch: u64,
    },
    GenesisKeyDelegation,
    MoveInstantaneousRewardsCert {
        from_reserves: bool,
        from_treasury: bool,
        to_stake_credentials: Option<Vec<(StakeCredential, i64)>>,
        to_other_pot: Option<u64>,
    },
    RollBack {
        block_slot: u64,
        block_hash: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub context: EventContext,

    #[serde(flatten)]
    pub data: EventData,

    pub fingerprint: Option<String>,
}
