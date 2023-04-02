//! Internal pipeline framework

use pallas::ledger::traverse::wellknown::GenesisValues;
use serde::Deserialize;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;

use pallas::network::miniprotocols::Point;
use pallas::network::upstream::cursor::Intersection;

pub mod errors;
pub mod legacy_v1;

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

pub type Cursor = pallas::network::upstream::cursor::Cursor;

pub struct Context {
    pub chain: GenesisValues,
    pub cursor: Cursor,
    pub error_policy: RuntimePolicy,
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
}

impl From<Record> for JsonValue {
    fn from(value: Record) -> Self {
        match value {
            Record::CborBlock(x) => json!({ "hex": hex::encode(x) }),
            Record::CborTx(x) => json!({ "hex": hex::encode(x) }),
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
            ChainEvent::Apply(x, _) => x,
            ChainEvent::Undo(x, _) => x,
            ChainEvent::Reset(x) => x,
        }
    }
}

pub type SourceOutputPort = gasket::messaging::crossbeam::OutputPort<ChainEvent>;
pub type FilterInputPort = gasket::messaging::crossbeam::InputPort<ChainEvent>;
pub type FilterOutputPort = gasket::messaging::crossbeam::OutputPort<ChainEvent>;
pub type MapperInputPort = gasket::messaging::crossbeam::InputPort<ChainEvent>;
pub type MapperOutputPort = gasket::messaging::crossbeam::OutputPort<ChainEvent>;
pub type SinkInputPort = gasket::messaging::crossbeam::InputPort<ChainEvent>;

pub type OutputAdapter = gasket::messaging::crossbeam::ChannelSendAdapter<ChainEvent>;
pub type InputAdapter = gasket::messaging::crossbeam::ChannelRecvAdapter<ChainEvent>;

pub trait StageBootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter);
    fn connect_input(&mut self, adapter: InputAdapter);
    fn spawn(self) -> Result<Vec<gasket::runtime::Tether>, Error>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(u64, String),
    Fallbacks(Vec<(u64, String)>),
}

impl IntersectConfig {
    pub fn get_point(&self) -> Option<Point> {
        match self {
            IntersectConfig::Point(slot, hash) => {
                let hash = hex::decode(hash).expect("valid hex hash");
                Some(Point::Specific(*slot, hash))
            }
            _ => None,
        }
    }

    pub fn get_fallbacks(&self) -> Option<Vec<Point>> {
        match self {
            IntersectConfig::Fallbacks(all) => {
                let mapped = all
                    .iter()
                    .map(|(slot, hash)| {
                        let hash = hex::decode(hash).expect("valid hex hash");
                        Point::Specific(*slot, hash)
                    })
                    .collect();

                Some(mapped)
            }
            _ => None,
        }
    }
}

impl From<IntersectConfig> for Intersection {
    fn from(value: IntersectConfig) -> Self {
        match value {
            IntersectConfig::Tip => Intersection::Tip,
            IntersectConfig::Origin => Intersection::Origin,
            IntersectConfig::Point(x, y) => {
                let point = Point::Specific(x, hex::decode(y).unwrap());

                Intersection::Breadcrumbs(VecDeque::from(vec![point]))
            }
            IntersectConfig::Fallbacks(x) => {
                let points: Vec<_> = x
                    .iter()
                    .map(|(x, y)| Point::Specific(*x, hex::decode(y).unwrap()))
                    .collect();

                Intersection::Breadcrumbs(VecDeque::from(points))
            }
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
