use std::ops::Deref;

use pallas::{
    codec::minicbor::decode,
    ledger::primitives::{alonzo, byron, probing, Era, ToHash},
    network::miniprotocols::Point,
};

use crate::Error;

pub(crate) struct CborHolder(Vec<u8>);

impl<'b> CborHolder {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn parse(&'b self) -> Result<MultiEraBlock<'b>, Error> {
        let block = match probing::probe_block_cbor_era(&self.0) {
            probing::Outcome::Matched(era) => match era {
                pallas::ledger::primitives::Era::Byron => {
                    let block = decode(&self.0)?;
                    MultiEraBlock::Byron(Box::new(block))
                }
                _ => {
                    let alonzo::BlockWrapper(_, block) = decode(&self.0)?;
                    MultiEraBlock::AlonzoCompatible(Box::new(block), era)
                }
            },
            // TODO: we're assuming that the geenesis block is Byron-compatible. Is this a safe
            // assumption?
            probing::Outcome::GenesisBlock => {
                let block = decode(&self.0)?;
                MultiEraBlock::Byron(Box::new(block))
            }
            probing::Outcome::Inconclusive => {
                log::error!("CBOR hex for debugging: {}", hex::encode(&self.0));
                return Err("can't infer primitive block from cbor, inconclusive probing".into());
            }
        };

        Ok(block)
    }

    pub fn cbor(&'b self) -> &'b [u8] {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) enum MultiEraBlock<'b> {
    Byron(Box<byron::Block>),
    AlonzoCompatible(Box<alonzo::Block<'b>>, Era),
}

impl MultiEraBlock<'_> {
    pub(crate) fn read_cursor(&self) -> Result<Point, Error> {
        match self {
            MultiEraBlock::Byron(x) => match x.deref() {
                byron::Block::EbBlock(x) => {
                    let hash = x.header.to_hash();
                    let slot = x.header.to_abs_slot();
                    Ok(Point::Specific(slot, hash.to_vec()))
                }
                byron::Block::MainBlock(x) => {
                    let hash = x.header.to_hash();
                    let slot = x.header.consensus_data.0.to_abs_slot();
                    Ok(Point::Specific(slot, hash.to_vec()))
                }
            },
            MultiEraBlock::AlonzoCompatible(x, _) => {
                let hash = x.header.to_hash();
                Ok(Point::Specific(x.header.header_body.slot, hash.to_vec()))
            }
        }
    }
}
