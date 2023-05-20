//! Internal pipeline framework

use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use std::fmt::Debug;
use std::path::PathBuf;

// we use UtxoRpc as our canonical representation of a parsed Tx
pub use utxorpc_spec_ledger::v1::Tx as ParsedTx;

// we use GenesisValues from Pallas as our ChainConfig
pub use pallas::ledger::traverse::wellknown::GenesisValues;

pub mod cursor;
pub mod errors;
pub mod legacy_v1;

pub use cursor::*;
pub use errors::*;

#[derive(Deserialize)]
#[serde(tag = "type")]
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

pub struct Context {
    pub chain: GenesisValues,
    pub intersect: IntersectConfig,
    pub cursor: Cursor,
    pub finalize: Option<FinalizeConfig>,
    pub current_dir: PathBuf,
}

use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone)]
pub enum Record {
    CborBlock(Vec<u8>),
    CborTx(Vec<u8>),
    GenericJson(JsonValue),
    OuraV1Event(legacy_v1::Event),
    ParsedTx(ParsedTx),
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::CborBlock(x) => json!({ "hex": hex::encode(x) }),
            Record::CborTx(x) => json!({ "hex": hex::encode(x) }),
            Record::ParsedTx(x) => json!(x),
            Record::OuraV1Event(x) => json!(x),
            Record::GenericJson(x) => x,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChainEvent {
    Apply(Point, Record),
    Undo(Point, Record),
    Reset(Point),
}

impl ChainEvent {
    pub fn apply(point: Point, record: impl Into<Record>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Apply(point, record.into()),
        }
    }

    pub fn undo(point: Point, record: impl Into<Record>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Undo(point, record.into()),
        }
    }

    pub fn reset(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Reset(point),
        }
    }

    pub fn point(&self) -> &Point {
        match self {
            Self::Apply(x, _) => x,
            Self::Undo(x, _) => x,
            Self::Reset(x) => x,
        }
    }

    pub fn record(&self) -> Option<&Record> {
        match self {
            Self::Apply(_, x) => Some(x),
            Self::Undo(_, x) => Some(x),
            _ => None,
        }
    }

    pub fn map_record(self, f: fn(Record) -> Record) -> Self {
        match self {
            Self::Apply(p, x) => Self::Apply(p, f(x)),
            Self::Undo(p, x) => Self::Undo(p, f(x)),
            Self::Reset(x) => Self::Reset(x),
        }
    }

    pub fn try_map_record<E>(self, f: fn(Record) -> Result<Record, E>) -> Result<Self, E> {
        let out = match self {
            Self::Apply(p, x) => Self::Apply(p, f(x)?),
            Self::Undo(p, x) => Self::Undo(p, f(x)?),
            Self::Reset(x) => Self::Reset(x),
        };

        Ok(out)
    }

    pub fn try_map_record_to_many<E>(
        self,
        f: fn(Record) -> Result<Vec<Record>, E>,
    ) -> Result<Vec<Self>, E> {
        let out = match self {
            Self::Apply(p, x) => f(x)?
                .into_iter()
                .map(|i| Self::Apply(p.clone(), i))
                .collect(),
            Self::Undo(p, x) => f(x)?
                .into_iter()
                .map(|i| Self::Undo(p.clone(), i))
                .collect(),
            Self::Reset(x) => vec![Self::Reset(x)],
        };

        Ok(out)
    }
}

fn point_to_json(point: Point) -> JsonValue {
    match &point {
        pallas::network::miniprotocols::Point::Origin => JsonValue::from("origin"),
        pallas::network::miniprotocols::Point::Specific(slot, hash) => {
            json!({ "slot": slot, "hash": hex::encode(hash)})
        }
    }
}

impl From<ChainEvent> for JsonValue {
    fn from(value: ChainEvent) -> Self {
        match value {
            ChainEvent::Apply(point, record) => {
                json!({
                    "event": "apply",
                    "point": point_to_json(point),
                    "record": JsonValue::from(record.clone())
                })
            }
            ChainEvent::Undo(point, record) => {
                json!({
                    "event": "undo",
                    "point": point_to_json(point),
                    "record": JsonValue::from(record.clone())
                })
            }
            ChainEvent::Reset(point) => {
                json!({
                    "event": "reset",
                    "point": point_to_json(point)
                })
            }
        }
    }
}

pub type SourceOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type FilterInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
pub type FilterOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type MapperInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
pub type MapperOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type SinkInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;

pub type OutputAdapter = gasket::messaging::tokio::ChannelSendAdapter<ChainEvent>;
pub type InputAdapter = gasket::messaging::tokio::ChannelRecvAdapter<ChainEvent>;

pub trait StageBootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter);
    fn connect_input(&mut self, adapter: InputAdapter);
    fn spawn(self, policy: gasket::runtime::Policy) -> gasket::runtime::Tether;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(u64, String),
    Breadcrumbs(Vec<(u64, String)>),
}

impl IntersectConfig {
    pub fn points(&self) -> Option<Vec<Point>> {
        match self {
            IntersectConfig::Breadcrumbs(all) => {
                let mapped = all
                    .iter()
                    .map(|(slot, hash)| {
                        let hash = hex::decode(hash).expect("valid hex hash");
                        Point::Specific(*slot, hash)
                    })
                    .collect();

                Some(mapped)
            }
            IntersectConfig::Point(slot, hash) => {
                let hash = hex::decode(hash).expect("valid hex hash");
                Some(vec![Point::Specific(*slot, hash)])
            }
            _ => None,
        }
    }
}

/// Optional configuration to stop processing new blocks after processing:
///   1. a block with the given hash
///   2. the first block on or after a given absolute slot
///   3. TODO: a total of X blocks
#[derive(Deserialize, Debug, Clone)]
pub struct FinalizeConfig {
    until_hash: Option<String>,
    max_block_slot: Option<u64>,
    // max_block_quantity: Option<u64>,
}

pub fn should_finalize(
    config: &Option<FinalizeConfig>,
    last_point: &Point,
    // block_count: u64,
) -> bool {
    let config = match config {
        Some(x) => x,
        None => return false,
    };

    if let Some(expected) = &config.until_hash {
        if let Point::Specific(_, current) = last_point {
            return expected == &hex::encode(current);
        }
    }

    if let Some(max) = config.max_block_slot {
        if last_point.slot_or_default() >= max {
            return true;
        }
    }

    // if let Some(max) = config.max_block_quantity {
    //     if block_count >= max {
    //         return true;
    //     }
    // }

    false
}
