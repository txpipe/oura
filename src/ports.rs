use pallas::ledger::alonzo::Value;

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub enum Event {
    Block {
        block_number: u64,
        slot: u64,
    },
    Transaction {
        fee: u64,
        ttl: Option<u64>,
        validity_interval_start: Option<u64>,
    },
    TxInput {
        transaction_id: String,
        index: u64,
    },
    TxOutput {
        address: String,
        amount: Value,
    },
    Metadata {
        key: String,
    },
    Mint {
        key1: String,
        key2: String,
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

pub trait InputPort {
    fn on_event(event: Event) -> Result<(), Error>;
}

pub trait OutputPort {
    fn get_next() -> Result<Event, Error>;
}
