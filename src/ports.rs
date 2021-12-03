use merge::Merge;
use pallas::ledger::alonzo::Value;

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Merge, Default)]
pub struct EventContext {
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub tx_id: Option<String>,
    pub input_idx: Option<usize>,
    pub output_idx: Option<usize>,
}

#[derive(Debug, Clone)]
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
        amount: Value,
    },
    OutputAsset {
        coin: u64,
        policy: String,
        asset: String,
        value: u64,
    },
    Metadata {
        key: String,
    },
    Mint {
        policy: String,
        asset: String,
        quantity: i64,
    },
    NativeScript,
    PlutusScript,
    StakeRegistration,
    StakeDeregistration,
    StakeDelegation,
    PoolRegistration,
    PoolRetirement,
    GenesisKeyDelegation,
    MoveInstantaneousRewardsCert,
}

pub struct Event {
    pub context: EventContext,
    pub data: EventData,
}