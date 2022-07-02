use pallas::{
    codec::minicbor::decode,
    ledger::primitives::{alonzo, byron, ToHash},
    ledger::traverse::{probe, Era},
    network::miniprotocols::Point,
};

use crate::Error;

pub(crate) struct CborHolder(Vec<u8>);

impl<'b> CborHolder {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn parse(&'b self) -> Result<MultiEraBlock<'b>, Error> {
        let block = match probe::block_era(&self.0) {
            probe::Outcome::Matched(era) => match era {
                Era::Byron => {
                    let block = decode(&self.0)?;
                    MultiEraBlock::Byron(Box::new(block))
                }
                Era::Shelley | Era::Allegra | Era::Mary | Era::Alonzo => {
                    let (_, block): (u16, alonzo::MintedBlock) = decode(&self.0)?;
                    MultiEraBlock::AlonzoCompatible(Box::new(block), era)
                }
                Era::Babbage => todo!(),
                _ => todo!(),
            },
            probe::Outcome::EpochBoundary => {
                let block = decode(&self.0)?;
                MultiEraBlock::EpochBoundary(Box::new(block))
            }
            probe::Outcome::Inconclusive => {
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
    EpochBoundary(Box<byron::EbBlock>),
    Byron(Box<byron::MintedBlock<'b>>),
    AlonzoCompatible(Box<alonzo::MintedBlock<'b>>, Era),
}

impl MultiEraBlock<'_> {
    pub(crate) fn read_cursor(&self) -> Result<Point, Error> {
        match self {
            MultiEraBlock::EpochBoundary(x) => {
                let hash = x.header.to_hash();
                let slot = x.header.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraBlock::Byron(x) => {
                let hash = x.header.to_hash();
                let slot = x.header.consensus_data.0.to_abs_slot();
                Ok(Point::Specific(slot, hash.to_vec()))
            }
            MultiEraBlock::AlonzoCompatible(x, _) => {
                let hash = x.header.to_hash();
                Ok(Point::Specific(x.header.header_body.slot, hash.to_vec()))
            }
        }
    }
}
