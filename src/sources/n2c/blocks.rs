use std::ops::Deref;

use pallas::{
    codec::minicbor::decode,
    ledger::primitives::{alonzo, byron, probing, Era},
    network::miniprotocols::{chainsync::BlockContent, Point},
};

use crate::Error;

#[derive(Debug)]
pub(crate) enum MultiEraBlock {
    Byron(Box<byron::Block>),
    AlonzoCompatible(Box<alonzo::Block>, Era),
}

impl TryFrom<BlockContent> for MultiEraBlock {
    type Error = Error;

    fn try_from(value: BlockContent) -> Result<Self, Self::Error> {
        let bytes = value.deref();

        match probing::probe_block_cbor_era(bytes) {
            probing::Outcome::Matched(era) => match era {
                pallas::ledger::primitives::Era::Byron => {
                    let block = decode(bytes)?;
                    Ok(MultiEraBlock::Byron(Box::new(block)))
                }
                _ => {
                    let alonzo::BlockWrapper(_, block) = decode(bytes)?;
                    Ok(MultiEraBlock::AlonzoCompatible(Box::new(block), era))
                }
            },
            // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
            // assumption?
            probing::Outcome::GenesisBlock => {
                let block = decode(bytes)?;
                Ok(MultiEraBlock::Byron(Box::new(block)))
            }
            probing::Outcome::Inconclusive => {
                log::error!("CBOR hex for debubbing: {}", hex::encode(bytes));
                Err("can't infer primitive block from cbor, inconslusive probing".into())
            }
        }
    }
}

impl MultiEraBlock {
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
                let hash = alonzo::crypto::hash_block_header(&x.header);
                Ok(Point::Specific(x.header.header_body.slot, hash.to_vec()))
            }
        }
    }
}
