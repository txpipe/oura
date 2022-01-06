use std::{
    collections::BTreeMap,
    fmt::Display,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use merge::Merge;

use pallas::ouroboros::network::handshake::{MAINNET_MAGIC, TESTNET_MAGIC};
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

use serde_json::Value as JsonValue;

use crate::mapping::MapperConfig;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInputRecord {
    pub tx_id: String,
    pub index: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputAssetRecord {
    pub policy: String,
    pub asset: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutputRecord {
    pub address: String,
    pub amount: u64,
    pub assets: Option<Vec<OutputAssetRecord>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MintRecord {
    pub policy: String,
    pub asset: String,
    pub quantity: i64,
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

pub type PartialBootstrapResult = Result<(JoinHandle<()>, Receiver<Event>), Error>;

pub type BootstrapResult = Result<JoinHandle<()>, Error>;

pub trait SourceConfig {
    fn bootstrap(&self) -> PartialBootstrapResult;
}

pub trait FilterConfig {
    fn bootstrap(&self, input: Receiver<Event>) -> PartialBootstrapResult;
}

pub trait SinkConfig {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult;
}

#[derive(Clone, Debug)]
pub struct EventWriter {
    context: EventContext,
    output: Sender<Event>,
    chain_info: Option<ChainWellKnownInfo>,
    pub mapping_config: MapperConfig,
}

impl EventWriter {
    pub fn new(
        output: Sender<Event>,
        chain_info: Option<ChainWellKnownInfo>,
        mapping_config: MapperConfig,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            chain_info,
            mapping_config,
        }
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        self.output.send(evt)?;

        Ok(())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
            chain_info: self.chain_info.clone(),
            mapping_config: self.mapping_config.clone(),
        }
    }

    pub fn compute_timestamp(&self, slot: u64) -> Option<u64> {
        self.chain_info
            .as_ref()
            .map(|info| info.shelley_known_time + (slot - info.shelley_known_slot))
    }
}

pub trait EventSource {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error>;
}
