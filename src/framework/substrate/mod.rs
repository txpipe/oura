use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

/// Serializable representation of a Substrate block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsics_root: String,
}

/// Signature information for signed extrinsics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub address: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<JsonValue>,
}

/// Call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    pub pallet_name: String,
    pub function_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<JsonValue>,
}

/// Events emitted by the extrinsic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrinsicEvent {
    pub pallet_name: String,
    pub event_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<JsonValue>,
}

/// Serializable representation of a Substrate extrinsic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extrinsic {
    pub index: u32,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,

    // Signature information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureInfo>,
}

/// Serializable representation of a complete Substrate block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedBlock {
    pub header: BlockHeader,
    pub extrinsics: Vec<Extrinsic>,
    pub extrinsics_count: usize,
}

#[derive(Debug, Clone)]
pub enum Record {
    ParsedBlock(Box<ParsedBlock>),
    RawBlock(Vec<u8>),
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::ParsedBlock(block) => json!(block),
            Record::RawBlock(bytes) => json!({ "hex": hex::encode(bytes) }),
        }
    }
}
