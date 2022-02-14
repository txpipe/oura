use pallas::{
    ledger::primitives::{alonzo, byron},
    network::miniprotocols::{chainsync::HeaderContent, Point},
};

use crate::Error;

#[derive(Debug)]
pub enum MultiEraHeader {
    ByronBoundary(byron::EbbHead),
    Byron(byron::BlockHead),
    Shelley(alonzo::Header),
}

impl TryFrom<HeaderContent> for MultiEraHeader {
    type Error = Error;

    fn try_from(value: HeaderContent) -> Result<Self, Self::Error> {
        match value {
            HeaderContent::Byron(variant, _, bytes) => match variant {
                0 => {
                    let header = minicbor::decode(&bytes)?;
                    Ok(MultiEraHeader::ByronBoundary(header))
                }
                _ => {
                    let header = minicbor::decode(&bytes)?;
                    Ok(MultiEraHeader::Byron(header))
                }
            },
            HeaderContent::Shelley(bytes) => {
                let header = minicbor::decode(&bytes)?;
                Ok(MultiEraHeader::Shelley(header))
            }
        }
    }
}

impl MultiEraHeader {
    pub fn read_cursor(&self) -> Result<Point, Error> {
        match self {
            MultiEraHeader::ByronBoundary(x) => {
                let hash = x.to_hash();
                let slot = x.to_abs_slot();
                Ok(Point(slot, hash.to_vec()))
            }
            MultiEraHeader::Byron(x) => {
                let hash = x.to_hash();
                let slot = x.consensus_data.0.to_abs_slot();
                Ok(Point(slot, hash.to_vec()))
            }
            MultiEraHeader::Shelley(x) => {
                let hash = alonzo::crypto::hash_block_header(x);
                Ok(Point(x.header_body.slot, hash.to_vec()))
            }
        }
    }
}
