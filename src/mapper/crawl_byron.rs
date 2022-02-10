use super::EventWriter;
use crate::{model::EventContext, Error};

use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{byron, Fragment};

impl EventWriter {
    fn crawl_byron_block(
        &self,
        block: &byron::Block,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<(), Error> {
        // TODO: actually crawl byron block
        match block {
            byron::Block::MainBlock(x) => {
                log::warn!(
                    "[not implemented] byron main block: {:?}",
                    x.header.consensus_data.0
                );
            }
            byron::Block::EbBlock(x) => {
                log::warn!(
                    "[not implemented] byron boundary block: {:?}",
                    x.header.consensus_data
                );
            }
        }

        Ok(())
    }

    pub fn crawl_from_byron_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let block = byron::Block::decode_fragment(cbor)?;

        let hash = byron::crypto::hash_block_header(&block);

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            // TODO
            //block_number: Some(block.header.header_body.block_number),
            //slot: Some(block.header.header_body.slot),
            //timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_byron_block(&block, &hash, cbor)
    }
}
