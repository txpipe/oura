use pallas::{
    ledger::primitives::{alonzo, byron, Fragment},
    network::miniprotocols::{
        chainsync, DecodePayload, EncodePayload, PayloadDecoder, PayloadEncoder, Point,
    },
};

use crate::Error;

#[derive(Debug)]
pub enum MultiEraHeader {
    ByronBoundary(byron::EbbHead),
    Byron(byron::BlockHead),
    Shelley(alonzo::Header),
}

impl EncodePayload for MultiEraHeader {
    fn encode_payload(&self, _e: &mut PayloadEncoder) -> Result<(), Error> {
        todo!()
    }
}

impl DecodePayload for MultiEraHeader {
    fn decode_payload(d: &mut PayloadDecoder) -> Result<Self, Error> {
        d.array()?;
        let variant = d.u32()?; // WTF is this value?

        match variant {
            // byron
            0 => {
                d.array()?;

                // can't find a reference anywhere about the structure of these values, but they
                // seem to provide the variant of header
                let (block_type, _): (u8, u64) = d.decode()?;

                d.tag()?;
                let bytes = d.bytes()?;

                match block_type {
                    0 => {
                        let header = byron::EbbHead::decode_fragment(bytes).unwrap();
                        Ok(MultiEraHeader::ByronBoundary(header))
                    }
                    _ => {
                        let header = byron::BlockHead::decode_fragment(bytes).unwrap();
                        Ok(MultiEraHeader::Byron(header))
                    }
                }
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

impl chainsync::BlockLike for MultiEraHeader {
    fn block_point(&self) -> Result<Point, Error> {
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
