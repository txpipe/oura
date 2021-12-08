use std::{
    ops::Deref,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventData {
    Block {
        body_size: usize,
        issuer_vkey: String,
    },
    Transaction {
        hash: Option<String>,
        fee: u64,
        ttl: Option<u64>,
        validity_interval_start: Option<u64>,
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
    NewNativeScript,
    NewPlutusScript {
        data: String,
    },
    PlutusScriptRef {
        data: String,
    },
    StakeRegistration,
    StakeDeregistration,
    StakeDelegation,
    PoolRegistration,
    PoolRetirement,
    GenesisKeyDelegation,
    MoveInstantaneousRewardsCert,
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

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl<T> ToHex for T
where
    T: Deref<Target = Vec<u8>>,
{
    fn to_hex(&self) -> String {
        hex::encode(self.deref())
    }
}

pub type Storage = Vec<Event>;

pub struct EventWriter<'a> {
    context: EventContext,
    storage: &'a mut Storage,
}

impl<'a> EventWriter<'a> {
    pub fn new(storage: &mut Storage) -> EventWriter<'_> {
        EventWriter {
            context: EventContext::default(),
            storage,
        }
    }

    pub fn append(&mut self, data: EventData) -> &mut Self {
        self.storage.push(Event {
            context: self.context.clone(),
            data,
        });

        self
    }

    pub fn child_writer(&mut self, mut extra_context: EventContext) -> EventWriter<'_> {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            storage: self.storage,
        }
    }
}

pub trait EventSource {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter);
}
