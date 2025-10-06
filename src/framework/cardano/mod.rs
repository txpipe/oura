use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

// we use UtxoRpc as our canonical representation of a parsed Tx
pub use pallas::interop::utxorpc::spec::cardano::Block as ParsedBlock;
pub use pallas::interop::utxorpc::spec::cardano::Tx as ParsedTx;

// we use GenesisValues from Pallas as our ChainConfig
pub use pallas::ledger::traverse::wellknown::GenesisValues;

pub mod legacy_v1;

#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChainConfig {
    Mainnet,
    Testnet,
    PreProd,
    Preview,
    Custom(GenesisValues),
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl From<ChainConfig> for GenesisValues {
    fn from(other: ChainConfig) -> Self {
        match other {
            ChainConfig::Mainnet => GenesisValues::mainnet(),
            ChainConfig::Testnet => GenesisValues::testnet(),
            ChainConfig::PreProd => GenesisValues::preprod(),
            ChainConfig::Preview => GenesisValues::preview(),
            ChainConfig::Custom(x) => x,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Record {
    CborBlock(Vec<u8>),
    CborTx(Vec<u8>),
    OuraV1Event(legacy_v1::Event),
    ParsedTx(ParsedTx),
    ParsedBlock(ParsedBlock),
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::CborBlock(x) => json!({ "hex": hex::encode(x) }),
            Record::CborTx(x) => json!({ "hex": hex::encode(x) }),
            Record::ParsedBlock(x) => json!(x),
            Record::ParsedTx(x) => json!(x),
            Record::OuraV1Event(x) => json!(x),
        }
    }
}
