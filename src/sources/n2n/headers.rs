use pallas::{
    codec::minicbor::decode,
    ledger::primitives::{alonzo, babbage, byron, ToHash},
    network::miniprotocols::{chainsync::HeaderContent, Point},
};

use crate::Error;

#[derive(Debug)]
pub enum MultiEraHeader {
    ByronBoundary(byron::EbbHead),
    Byron(byron::BlockHead),
    AlonzoCompatible(alonzo::Header),
    Babbage(babbage::Header),
}

impl TryFrom<HeaderContent> for MultiEraHeader {
    type Error = Error;

    fn try_from(value: HeaderContent) -> Result<Self, Self::Error> {
        match value.variant {
            0 => match value.byron_prefix {
                Some((0, _)) => {
                    let header = decode(&value.cbor)?;
                    Ok(MultiEraHeader::ByronBoundary(header))
                }
                _ => {
                    let header = decode(&value.cbor)?;
                    Ok(MultiEraHeader::Byron(header))
                }
            },
            1 | 2 | 3 | 4 => {
                let header = decode(&value.cbor)?;
                Ok(MultiEraHeader::AlonzoCompatible(header))
            }
            5 => {
                let header = decode(&value.cbor)?;
                Ok(MultiEraHeader::Babbage(header))
            }
            x => Err(format!("This version of Oura can't handle era: {}", x).into()),
        }
    }
}

impl MultiEraHeader {
    pub fn read_cursor(&self) -> Result<Point, Error> {
        match self {
            MultiEraHeader::ByronBoundary(x) => {
                let hash = x.to_hash();
                let slot = x.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraHeader::Byron(x) => {
                let hash = x.to_hash();
                let slot = x.consensus_data.0.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraHeader::AlonzoCompatible(x) => {
                let hash = x.to_hash();
                Ok(Point::Specific(x.header_body.slot, hash.to_vec()))
            }
            MultiEraHeader::Babbage(x) => {
                let hash = x.to_hash();
                Ok(Point::Specific(x.header_body.slot, hash.to_vec()))
            }
        }
    }
}
