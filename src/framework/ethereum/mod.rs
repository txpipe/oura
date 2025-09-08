use alloy::rpc::types::Block;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone)]
pub enum Record {
    ParsedBlock(Block),
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::ParsedBlock(x) => json!(x),
        }
    }
}
