use std::{
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use merge::Merge;

use serde_derive::{Deserialize, Serialize};

pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Debug, Clone, Merge, Default)]
pub struct EventContext {
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub tx_idx: Option<usize>,
    pub tx_hash: Option<String>,
    pub input_idx: Option<usize>,
    pub output_idx: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StakeCredential {
    AddrKeyhash(String),
    Scripthash(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventData {
    Block {
        body_size: usize,
        issuer_vkey: String,
    },
    Transaction {
        fee: u64,
        ttl: Option<u64>,
        validity_interval_start: Option<u64>,
        network_id: Option<u32>,
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
    NewNativeScript,
    NewPlutusScript {
        data: String,
    },
    PlutusScriptRef {
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
    pub data: EventData,
}

pub type BootstrapResult = Result<JoinHandle<()>, Error>;

pub trait SourceConfig {
    fn bootstrap(&self, output: Sender<Event>) -> BootstrapResult;
}

pub trait SinkConfig {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult;
}

#[derive(Debug)]
pub struct EventWriter {
    context: EventContext,
    output: Sender<Event>,
}

impl EventWriter {
    pub fn new(output: Sender<Event>) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
        }
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
        };

        self.output.send(evt)?;

        Ok(())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
        }
    }
}

pub trait EventSource {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error>;
}
