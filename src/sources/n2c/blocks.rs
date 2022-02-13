use std::ops::Deref;

use pallas::{
    ledger::primitives::{alonzo, byron, probing, Fragment},
    network::miniprotocols::{
        chainsync, DecodePayload, EncodePayload, PayloadDecoder, PayloadEncoder, Point,
    },
};

use crate::Error;

#[derive(Debug)]
pub(crate) enum MultiEraBlock {
    Byron(Box<byron::Block>, Vec<u8>),
    Shelley(Box<alonzo::Block>, Vec<u8>),
}

impl EncodePayload for MultiEraBlock {
    fn encode_payload(&self, _e: &mut PayloadEncoder) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

impl DecodePayload for MultiEraBlock {
    fn decode_payload(d: &mut PayloadDecoder) -> Result<Self, Box<dyn std::error::Error>> {
        d.tag()?;
        let bytes = d.bytes()?;

        match probing::probe_block_cbor(bytes) {
            probing::BlockInference::Byron => {
                let block = byron::Block::decode_fragment(bytes)?;
                Ok(MultiEraBlock::Byron(Box::new(block), Vec::from(bytes)))
            }
            probing::BlockInference::Shelley => {
                let alonzo::BlockWrapper(_, block) = alonzo::BlockWrapper::decode_fragment(bytes)?;
                Ok(MultiEraBlock::Shelley(Box::new(block), Vec::from(bytes)))
            }
            probing::BlockInference::Inconclusive => {
                log::error!("CBOR hex for debubbing: {}", hex::encode(bytes));
                Err("can't infer primitive block from cbor, inconslusive probing".into())
            }
        }
    }
}

impl chainsync::BlockLike for MultiEraBlock {
    fn block_point(&self) -> Result<Point, Error> {
        match self {
            MultiEraBlock::Byron(x, _) => match x.deref() {
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
            MultiEraBlock::Shelley(x, _) => {
                let hash = alonzo::crypto::hash_block_header(&x.header);
                Ok(Point(x.header.header_body.slot, hash.to_vec()))
            }
        }
    }
}
