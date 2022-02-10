use pallas::{
    ledger::primitives::{alonzo, byron, Fragment},
    network::miniprotocols::{
        chainsync, DecodePayload, EncodePayload, PayloadDecoder, PayloadEncoder, Point,
    },
};

use crate::Error;

#[derive(Debug)]
pub enum MultiEraHeader {
    Byron(byron::BlockHead),
    Shelley(alonzo::Header),
}

impl EncodePayload for MultiEraHeader {
    fn encode_payload(&self, e: &mut PayloadEncoder) -> Result<(), Error> {
        todo!()
    }
}

impl DecodePayload for MultiEraHeader {
    fn decode_payload(d: &mut PayloadDecoder) -> Result<Self, Error> {
        d.array()?;
        let variant = d.u32()?; // WTF is this value?

        log::info!("found multi-era header variant: {}", variant);

        match variant {
            // byron
            0 => {
                d.array()?;

                // not sure what these values are, can't find a reference anywhere
                let _mystery_values: (u8, u64) = d.decode()?;

                d.tag()?;
                let bytes = d.bytes()?;

                let header = byron::BlockHead::decode_fragment(bytes).unwrap();

                Ok(MultiEraHeader::Byron(header))
            }
            // shelley
            _ => {
                d.tag()?;
                let bytes = d.bytes()?;
                let header = alonzo::Header::decode_fragment(bytes)?;

                Ok(MultiEraHeader::Shelley(header))
            }
        }
    }
}

// TODO: un-hardcode these values
const BYRON_SLOT_LENGTH: u64 = 20; // seconds
const BYRON_EPOCH_LENGTH: u64 = 5 * 24 * 60 * 60; // 5 days

fn byron_sub_epoch_slot_to_absolute(epoch: u64, sub_epoch_slot: u64) -> u64 {
    ((epoch * BYRON_EPOCH_LENGTH) / BYRON_SLOT_LENGTH) + sub_epoch_slot
}

impl chainsync::BlockLike for MultiEraHeader {
    fn block_point(&self) -> Result<Point, Error> {
        match self {
            MultiEraHeader::Byron(x) => {
                let hash = byron::crypto::hash_main_block_header(x);

                let slot = byron_sub_epoch_slot_to_absolute(
                    x.consensus_data.0.epoch,
                    x.consensus_data.0.slot,
                );

                Ok(Point(slot, hash.to_vec()))
            }
            MultiEraHeader::Shelley(x) => {
                let hash = alonzo::crypto::hash_block_header(&x);
                Ok(Point(x.header_body.slot, hash.to_vec()))
            }
        }
    }
}
