use pallas::{
    ledger::primitives::{alonzo, byron, Fragment},
    network::miniprotocols::{
        chainsync, DecodePayload, EncodePayload, PayloadDecoder, PayloadEncoder, Point,
    },
};

use crate::Error;

#[derive(Debug)]
pub enum MultiEraHeader {
    ByronMainBlock(byron::BlockHead),
    ByronEbbBlock(byron::EbbHead),
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
            // byron main block
            0 => {
                d.array()?;
                d.skip()?;

                d.tag()?;
                let bytes = d.bytes()?;

                let header = byron::BlockHead::decode_fragment(bytes)?;

                Ok(MultiEraHeader::ByronMainBlock(header))
            }
            1 => {
                d.tag()?;
                let bytes = d.bytes()?;

                // TODO: this fails, cbor structure doesn't match a boundary block header, still
                // investigating
                let header = byron::EbbHead::decode_fragment(bytes).unwrap();

                Ok(MultiEraHeader::ByronEbbBlock(header))
            }
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
            MultiEraHeader::ByronMainBlock(x) => {
                let hash = byron::crypto::hash_main_block_header(x);

                let slot = byron_sub_epoch_slot_to_absolute(
                    x.consensus_data.0.epoch,
                    x.consensus_data.0.slot,
                );

                Ok(Point(slot, hash.to_vec()))
            }
            MultiEraHeader::ByronEbbBlock(x) => {
                let hash = byron::crypto::hash_boundary_block_header(x);

                let slot = byron_sub_epoch_slot_to_absolute(x.consensus_data.epoch_id, 0);
                let point = Point(slot, hash.to_vec());

                log::warn!("{:?}", point);

                Ok(point)
            }
            MultiEraHeader::Shelley(x) => {
                let hash = alonzo::crypto::hash_block_header(&x);
                Ok(Point(x.header_body.slot, hash.to_vec()))
            }
        }
    }
}
