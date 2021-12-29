use std::{
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use merge::Merge;

use pallas::ouroboros::network::handshake::{MAINNET_MAGIC, TESTNET_MAGIC};
use serde_derive::{Deserialize, Serialize};
use strum_macros::Display;

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
    Transaction {
        fee: u64,
        ttl: Option<u64>,
        validity_interval_start: Option<u64>,
        network_id: Option<u32>,
        input_count: usize,
        output_count: usize,
        total_output: u64,
    },
    TxInput {
        tx_id: String,
        index: u64,
    },
    TxOutput {
        address: String,
        amount: u64,
    },
    OutputAsset {
        policy: String,
        asset: String,
        amount: u64,
    },
    Metadata {
        key: String,
        subkey: Option<String>,
        // TODO: value should be some sort of structured, JSON-like value.
        // we could use Pallas' Metadatum struct, but it needs to be clonable
        value: Option<String>,
    },
    Mint {
        policy: String,
        asset: String,
        quantity: i64,
    },
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

#[derive(Debug)]
pub struct EventWriter {
    context: EventContext,
    output: Sender<Event>,
    chain_info: Option<ChainWellKnownInfo>,
}

impl EventWriter {
    pub fn new(output: Sender<Event>, chain_info: Option<ChainWellKnownInfo>) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            chain_info,
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
