use merge::Merge;
use serde_derive::{Serialize, Deserialize};

pub type Error = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Debug, Clone, Merge, Default)]
pub struct EventContext {
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub tx_idx: Option<usize>,
    pub tx_id: Option<String>,
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
