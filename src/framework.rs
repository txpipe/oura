use std::{collections::BTreeMap, fmt::Display, thread::JoinHandle};

use merge::Merge;

use pallas::ouroboros::network::handshake::{MAINNET_MAGIC, TESTNET_MAGIC};
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use serde_json::Value as JsonValue;

pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
}

impl ChainWellKnownInfo {
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_known_slot: 4492799,
                shelley_known_hash:
                    "f8084c61b6a238acec985b59310b6ecec49c0ab8352249afd7268da5cff2a457".to_string(),
                shelley_known_time: 1596059071,
            }),
            TESTNET_MAGIC => Ok(ChainWellKnownInfo {
                shelley_known_slot: 1598399,
                shelley_known_hash:
                    "7e16781b40ebf8b6da18f7b5e8ade855d6738095ef2f1c58c77e88b6e45997a4".to_string(),
                shelley_known_time: 1595967596,
            }),
            _ => Err("can't infer well-known chain infro from specified magic".into()),
        }
    }
}

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
    pub policy: String,
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub image: JsonValue,
    pub media_type: Option<String>,
    pub description: Option<JsonValue>,
    pub raw_json: Option<JsonValue>,
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

#[derive(Serialize, Deserialize, Display, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventData {
    Block {
        body_size: usize,
        issuer_vkey: String,
        tx_count: usize,
    },
    Transaction(TransactionRecord),
    TxInput(TxInputRecord),
    TxOutput(TxOutputRecord),
    OutputAsset(OutputAssetRecord),
    Metadata(MetadataRecord),
    CIP25Asset(CIP25AssetRecord),
    Mint(MintRecord),
    Collateral {
        tx_id: String,
        index: u64,
    },
    NativeScript,
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
        to_stake_credentials: Option<BTreeMap<StakeCredential, i64>>,
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

pub type PartialBootstrapResult = Result<(JoinHandle<()>, StageReceiver), Error>;

pub type BootstrapResult = Result<JoinHandle<()>, Error>;

pub trait SourceConfig {
    fn bootstrap(&self) -> PartialBootstrapResult;
}

pub trait FilterConfig {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult;
}

pub trait SinkConfig {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult;
}

pub type StageReceiver = std::sync::mpsc::Receiver<Event>;

pub type StageSender = std::sync::mpsc::SyncSender<Event>;

/// The amount of events an inter-stage channel can buffer before blocking
///
/// If a filter or sink has a consumption rate lower than the rate of event
/// generations from a source, the pending events will buffer in a queue
/// provided by the corresponding mpsc channel implementation. This constant
/// defines the max amount of events that the buffer queue can hold. Once
/// reached, the previous stages in the pipeline will start blockin on 'send'.
///
/// This value has a direct effect on the amount of memory consumed by the
/// process. The higher the buffer, the higher potential memory consumption.
///
/// This value has a direct effect on performance. To allow _pipelining_
/// benefits, stages should be allowed certain degree of flexibility to deal
/// with resource constrains (such as network or cpu). The lower the buffer, the
/// lower degree of flexibility.
const DEFAULT_INTER_STAGE_BUFFER_SIZE: usize = 1000;

pub type StageChannel = (StageSender, StageReceiver);

/// Centralizes the implementation details of inter-stage channel creation
///
/// Concrete channel implementation is subject to change. We're still exploring
/// sync vs unbounded and threaded vs event-loop. Until we have a long-term
/// strategy, it makes sense to have a single place in the codebase that can be
/// used to change from one implementation to the other without incurring on
/// heavy refactoring throughout several files.
///
/// Sometimes centralization is not such a bad thing :)
pub fn new_inter_stage_channel(buffer_size: Option<usize>) -> StageChannel {
    std::sync::mpsc::sync_channel(buffer_size.unwrap_or(DEFAULT_INTER_STAGE_BUFFER_SIZE))
}
