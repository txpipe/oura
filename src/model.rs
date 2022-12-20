use pallas::network::miniprotocols::Point;
use std::fmt::Debug;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum RawBlockPayload {
    RollForward(Vec<u8>),
    RollBack(Point),
}

impl RawBlockPayload {
    pub fn roll_forward(block: Vec<u8>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(block),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}
