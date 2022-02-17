use std::ops::Deref;

use pallas::{
    ledger::primitives::{alonzo, byron, probing},
    network::miniprotocols::{chainsync::BlockContent, Point},
};

use crate::Error;

#[derive(Debug)]
pub(crate) enum MultiEraBlock {
    Byron(Box<byron::Block>),
    Shelley(Box<alonzo::Block>),
}

impl TryFrom<BlockContent> for MultiEraBlock {
    type Error = Error;

    fn try_from(value: BlockContent) -> Result<Self, Self::Error> {
        let bytes = value.deref();

        match probing::probe_block_cbor(bytes) {
            probing::BlockInference::Byron => {
                let block = minicbor::decode(bytes)?;
                Ok(MultiEraBlock::Byron(Box::new(block)))
            }
            probing::BlockInference::Shelley => {
                let alonzo::BlockWrapper(_, block) = minicbor::decode(bytes)?;
                Ok(MultiEraBlock::Shelley(Box::new(block)))
            }
            probing::BlockInference::Inconclusive => {
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
                    Ok(Point(slot, hash.to_vec()))
                }
                byron::Block::MainBlock(x) => {
                    let hash = x.header.to_hash();
                    let slot = x.header.consensus_data.0.to_abs_slot();
                    Ok(Point(slot, hash.to_vec()))
                }
            },
            MultiEraBlock::Shelley(x) => {
                let hash = alonzo::crypto::hash_block_header(&x.header);
                Ok(Point(x.header.header_body.slot, hash.to_vec()))
            }
        }
    }
}
