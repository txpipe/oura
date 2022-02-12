use std::ops::Deref;

use super::map::ToHex;
use super::EventWriter;
use crate::model::{BlockRecord, EventData, TransactionRecord, TxInputRecord, TxOutputRecord};
use crate::{model::EventContext, Error};

use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{byron, Fragment};

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
        source: &byron::TxPayload,
        tx_hash: &str,
    ) -> Result<TransactionRecord, Error> {
        let input_records = self.collect_byron_input_records(&source.transaction);
        let output_records = self.collect_byron_output_records(&source.transaction)?;

        let mut record = TransactionRecord {
            hash: tx_hash.to_owned(),
            // TODO: we have a problem with here. AFAIK, there's no reference to the tx fee in the
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

    fn crawl_byron_transaction(
        &self,
        source: &byron::TxPayload,
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
        source: &byron::MainBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<BlockRecord, Error> {
        Ok(BlockRecord {
            body_size: cbor.len() as usize,
            issuer_vkey: source.header.consensus_data.1.to_hex(),
            tx_count: source.body.tx_payload.len(),
            hash: hash.to_hex(),
            number: source.header.consensus_data.2[0],
            slot: source.header.consensus_data.0.to_abs_slot(),
            previous_hash: source.header.prev_block.to_hex(),
            cbor_hex: match self.config.include_block_cbor {
                true => hex::encode(cbor).into(),
                false => None,
            },
        })
    }

    fn crawl_byron_block(
        &self,
        block: &byron::MainBlock,
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

    pub fn crawl_from_byron_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let block = byron::Block::decode_fragment(cbor)?;

        if let byron::Block::MainBlock(block) = block {
            let hash = &block.header.to_hash();

            let child = self.child_writer(EventContext {
                block_hash: Some(hex::encode(&hash)),
                block_number: Some(block.header.consensus_data.2[0]),
                slot: Some(block.header.consensus_data.0.to_abs_slot()),
                //timestamp: self.compute_timestamp(block.header.header_body.slot),
                ..EventContext::default()
            });

            child.crawl_byron_block(&block, hash, cbor)?;
        }

        Ok(())
    }
}
