use std::ops::Deref;

use super::map::ToHex;
use super::EventWriter;
use crate::model::{BlockRecord, Era, EventData, TransactionRecord, TxInputRecord, TxOutputRecord};
use crate::{model::EventContext, Error};

use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{byron, ToHash};

impl EventWriter {
    fn to_byron_input_record(&self, source: &byron::TxIn) -> Option<TxInputRecord> {
        match source {
            byron::TxIn::Variant0(x) => {
                let (hash, index) = x.deref();

                Some(TxInputRecord {
                    tx_id: hash.to_hex(),
                    index: *index as u64,
                })
            }
            byron::TxIn::Other(a, b) => {
                log::warn!(
                    "don't know how to handle byron input: ({}, {})",
                    a,
                    b.to_hex()
                );

                None
            }
        }
    }

    fn collect_byron_input_records(&self, source: &byron::Tx) -> Vec<TxInputRecord> {
        source
            .inputs
            .iter()
            .filter_map(|x| self.to_byron_input_record(x))
            .collect()
    }

    fn to_byron_output_record(&self, source: &byron::TxOut) -> Result<TxOutputRecord, Error> {
        Ok(TxOutputRecord {
            address: source.address.to_addr_string()?,
            amount: source.amount,
            assets: None,
            datum_hash: None,
        })
    }

    fn collect_byron_output_records(
        &self,
        source: &byron::Tx,
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .outputs
            .iter()
            .map(|x| self.to_byron_output_record(x))
            .collect()
    }

    fn to_byron_transaction_record(
        &self,
        source: &byron::MintedTxPayload,
        tx_hash: &str,
    ) -> Result<TransactionRecord, Error> {
        let input_records = self.collect_byron_input_records(&source.transaction);
        let output_records = self.collect_byron_output_records(&source.transaction)?;

        let mut record = TransactionRecord {
            hash: tx_hash.to_owned(),
            // TODO: we have a problem here. AFAIK, there's no reference to the tx fee in the
            // block contents. This leaves us with the two alternative: a) compute the value, b)
            // omit the value.
            //
            // Computing the value is not trivial, the linear policy is easy to
            // implement, but tracking the parameters for each epoch means hardcoding values or
            // doing some extra queries.
            //
            // Ommiting the value elegantly would require turning the property data type into an
            // option, which is a breaking change.
            //
            // Chossing the lesser evil, going to send a `0` in the field and add a comment to the
            // docs notifying about this as a known issue to be fixed in v2.

            //fee: source.compute_fee_with_defaults()?,
            fee: 0,
            size: (source.transaction.raw_cbor().len() + source.witness.raw_cbor().len()) as u32,
            input_count: input_records.len(),
            output_count: output_records.len(),
            total_output: output_records.iter().map(|o| o.amount).sum(),
            ..Default::default()
        };

        if self.config.include_transaction_details {
            record.inputs = input_records.into();
            record.outputs = output_records.into();
        }

        Ok(record)
    }

    pub fn collect_byron_tx_records(
        &self,
        block: &byron::MintedBlock,
    ) -> Result<Vec<TransactionRecord>, Error> {
        block
            .body
            .tx_payload
            .iter()
            .map(|tx| {
                let tx_hash = tx.transaction.to_hash().to_string();
                self.to_byron_transaction_record(tx, &tx_hash)
            })
            .collect()
    }

    fn crawl_byron_transaction(
        &self,
        source: &byron::MintedTxPayload,
        tx_hash: &str,
    ) -> Result<(), Error> {
        let record = self.to_byron_transaction_record(source, tx_hash)?;

        self.append_from(record.clone())?;

        for (idx, input) in source.transaction.inputs.iter().enumerate() {
            let child = self.child_writer(EventContext {
                input_idx: Some(idx),
                ..EventContext::default()
            });

            if let Some(record) = self.to_byron_input_record(input) {
                child.append_from(record)?;
            }
        }

        for (idx, output) in source.transaction.outputs.iter().enumerate() {
            let child = self.child_writer(EventContext {
                output_idx: Some(idx),
                ..EventContext::default()
            });

            if let Ok(record) = self.to_byron_output_record(output) {
                child.append_from(record)?;
            }
        }

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    pub fn to_byron_block_record(
        &self,
        source: &byron::MintedBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<BlockRecord, Error> {
        let mut record = BlockRecord {
            era: Era::Byron,
            body_size: cbor.len() as usize,
            issuer_vkey: source.header.consensus_data.1.to_hex(),
            tx_count: source.body.tx_payload.len(),
            hash: hash.to_hex(),
            number: source.header.consensus_data.2[0],
            slot: source.header.consensus_data.0.to_abs_slot(),
            epoch: Some(source.header.consensus_data.0.epoch),
            epoch_slot: Some(source.header.consensus_data.0.slot),
            previous_hash: source.header.prev_block.to_hex(),
            cbor_hex: match self.config.include_block_cbor {
                true => hex::encode(cbor).into(),
                false => None,
            },
            transactions: None,
        };

        if self.config.include_block_details {
            record.transactions = Some(self.collect_byron_tx_records(source)?);
        }

        Ok(record)
    }

    fn crawl_byron_main_block(
        &self,
        block: &byron::MintedBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<(), Error> {
        let record = self.to_byron_block_record(block, hash, cbor)?;

        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.body.tx_payload.iter().enumerate() {
            let tx_hash = tx.transaction.to_hash().to_string();

            let child = self.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: Some(tx_hash.to_owned()),
                ..EventContext::default()
            });

            child.crawl_byron_transaction(tx, &tx_hash)?;
        }

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    pub fn to_byron_epoch_boundary_record(
        &self,
        source: &byron::EbBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<BlockRecord, Error> {
        Ok(BlockRecord {
            era: Era::Byron,
            body_size: cbor.len() as usize,
            hash: hash.to_hex(),
            issuer_vkey: Default::default(),
            tx_count: 0,
            number: source.header.consensus_data.difficulty[0],
            slot: source.header.to_abs_slot(),
            epoch: Some(source.header.consensus_data.epoch_id),
            epoch_slot: Some(0),
            previous_hash: source.header.prev_block.to_hex(),
            cbor_hex: match self.config.include_block_cbor {
                true => hex::encode(cbor).into(),
                false => None,
            },
            transactions: None,
        })
    }

    fn crawl_byron_ebb_block(
        &self,
        block: &byron::EbBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<(), Error> {
        let record = self.to_byron_epoch_boundary_record(block, hash, cbor)?;

        self.append_from(record.clone())?;

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    /// Mapper entry-point for decoded Byron blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we already have a decoded block (for example, N2C). The raw CBOR is also
    /// passed through in case we need to attach it to outbound events.
    pub fn crawl_byron_with_cbor(
        &self,
        block: &byron::MintedBlock,
        cbor: &[u8],
    ) -> Result<(), Error> {
        let hash = block.header.to_hash();
        let abs_slot = block.header.consensus_data.0.to_abs_slot();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            block_number: Some(block.header.consensus_data.2[0]),
            slot: Some(abs_slot),
            timestamp: self.compute_timestamp(abs_slot),
            ..EventContext::default()
        });

        child.crawl_byron_main_block(block, &hash, cbor)?;

        Ok(())
    }

    /// Mapper entry-point for raw Byron cbor blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we haven't decoded the CBOR yet (for example, N2N).
    pub fn crawl_from_byron_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let (_, block): (u16, byron::MintedBlock) = pallas::codec::minicbor::decode(cbor)?;
        self.crawl_byron_with_cbor(&block, cbor)
    }

    /// Mapper entry-point for decoded Byron Epoch-Boundary blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we already have a decoded block (for example, N2C). The raw CBOR is also
    /// passed through in case we need to attach it to outbound events.
    pub fn crawl_ebb_with_cbor(&self, block: &byron::EbBlock, cbor: &[u8]) -> Result<(), Error> {
        if self.config.include_byron_ebb {
            let hash = block.header.to_hash();
            let abs_slot = block.header.to_abs_slot();

            let child = self.child_writer(EventContext {
                block_hash: Some(hex::encode(&hash)),
                block_number: Some(block.header.consensus_data.difficulty[0]),
                slot: Some(abs_slot),
                timestamp: self.compute_timestamp(abs_slot),
                ..EventContext::default()
            });

            child.crawl_byron_ebb_block(block, &hash, cbor)?;
        }

        Ok(())
    }

    /// Mapper entry-point for raw EBB cbor blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we haven't decoded the CBOR yet (for example, N2N).
    pub fn crawl_from_ebb_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let (_, block): (u16, byron::EbBlock) = pallas::codec::minicbor::decode(cbor)?;
        self.crawl_ebb_with_cbor(&block, cbor)
    }
}
