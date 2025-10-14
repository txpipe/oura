use corepc_types::bitcoin::Block;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone)]
pub enum Record {
    // Scaffold placeholder for now
    ParsedBlock(Box<Block>),
    RawBlock(Vec<u8>),
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::ParsedBlock(x) => json!(x),
            Record::RawBlock(x) => json!({ "hex": hex::encode(x) }),
        }
    }
}
