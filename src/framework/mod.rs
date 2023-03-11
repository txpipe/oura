//! Internal pipeline framework

pub mod legacy_v1;

use serde::Deserialize;
use std::fmt::Debug;
use thiserror::Error;

use pallas::network::miniprotocols::Point;
use pallas::network::upstream::chains::WellKnownChainInfo;
use pallas::network::upstream::cursor::Cursor;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ChainConfig {
    Mainnet,
    Testnet,
    PreProd,
    Preview,
    Custom(WellKnownChainInfo),
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl From<ChainConfig> for WellKnownChainInfo {
    fn from(other: ChainConfig) -> Self {
        match other {
            ChainConfig::Mainnet => WellKnownChainInfo::mainnet(),
            ChainConfig::Testnet => WellKnownChainInfo::testnet(),
            ChainConfig::PreProd => WellKnownChainInfo::preprod(),
            ChainConfig::Preview => WellKnownChainInfo::preview(),
            ChainConfig::Custom(x) => x,
        }
    }
}

pub struct Context {
    pub chain: WellKnownChainInfo,
    pub cursor: Cursor,
}

use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub enum Record {
    CborBlock(Vec<u8>),
    CborTx(Vec<u8>),
    GenericJson(JsonValue),
    OuraV1Event(legacy_v1::Event),
}

impl From<Vec<u8>> for Record {
    fn from(value: Vec<u8>) -> Self {
        Record::CborBlock(value)
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
}

pub type SourceOutputPort = gasket::messaging::OutputPort<ChainEvent>;
pub type FilterInputPort = gasket::messaging::InputPort<ChainEvent>;
pub type FilterOutputPort = gasket::messaging::OutputPort<ChainEvent>;
pub type MapperInputPort = gasket::messaging::InputPort<ChainEvent>;
pub type MapperOutputPort = gasket::messaging::OutputPort<ChainEvent>;
pub type SinkInputPort = gasket::messaging::InputPort<ChainEvent>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error")]
    Config(String),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    pub fn config(err: impl ToString) -> Self {
        Self::Config(err.to_string())
    }

    pub fn custom(err: impl ToString) -> Self {
        Self::Custom(err.to_string())
    }
}
