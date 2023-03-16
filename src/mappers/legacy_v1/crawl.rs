use pallas::ledger::primitives::babbage::MintedDatumOption;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraInput, MultiEraOutput, MultiEraTx};
use pallas::network::miniprotocols::Point;

use crate::framework::legacy_v1::*;
use crate::framework::AppliesPolicy;
use crate::framework::Error as OuraError;

use gasket::error::Error;

use super::EventWriter;

impl From<pallas::ledger::traverse::Era> for Era {
    fn from(other: pallas::ledger::traverse::Era) -> Self {
        match other {
            pallas::ledger::traverse::Era::Byron => Era::Byron,
            pallas::ledger::traverse::Era::Shelley => Era::Shelley,
            pallas::ledger::traverse::Era::Allegra => Era::Allegra,
            pallas::ledger::traverse::Era::Mary => Era::Mary,
            pallas::ledger::traverse::Era::Alonzo => Era::Alonzo,
            pallas::ledger::traverse::Era::Babbage => Era::Babbage,
            _ => Era::Unknown,
        }
    }
}

impl EventWriter<'_> {
    fn crawl_collateral(&mut self, collateral: &MultiEraInput) -> Result<(), Error> {
        self.append(self.to_collateral_event(collateral))

        // TODO: should we have a collateral idx in context?
        // more complex event goes here (eg: ???)
    }

    fn crawl_metadata(&mut self, tx: &MultiEraTx) -> Result<(), Error> {
        let metadata = tx.metadata();
        let metadata = metadata.collect::<Vec<_>>();

        for (label, content) in metadata.iter() {
            let record = self.to_metadata_record(label, content);
            self.append_from(record)?;

            match label {
                721u64 => self.crawl_metadata_label_721(content)?,
                61284u64 => self.crawl_metadata_label_61284(content)?,
                _ => (),
            }
        }

        Ok(())
    }

    fn crawl_transaction_output(&mut self, output: &MultiEraOutput) -> Result<(), Error> {
        let record = self.to_transaction_output_record(output);
        self.append(record.into())?;

        let address = output
            .address()
            .map_err(OuraError::parse)
            .apply_policy(self.error_policy)
            .unwrap();

        let mut child = self.child_writer(EventContext {
            output_address: address.map(|x| x.to_string()),
            ..EventContext::default()
        });

        for asset in output.assets() {
            child.append_from(self.to_transaction_output_asset_record(&asset))?;
        }

        if let Some(MintedDatumOption::Data(datum)) = &output.datum() {
            child.append_from(self.to_plutus_datum_record(datum))?;
        }

        Ok(())
    }

    fn crawl_witnesses(&mut self, tx: &MultiEraTx) -> Result<(), Error> {
        for script in tx.native_scripts() {
            self.append_from(self.to_native_witness_record(script))?;
        }

        for script in tx.plutus_v1_scripts() {
            self.append_from(self.to_plutus_v1_witness_record(script))?;
        }

        for script in tx.plutus_v2_scripts() {
            self.append_from(self.to_plutus_v2_witness_record(script))?;
        }

        for redeemer in tx.redeemers() {
            self.append_from(self.to_plutus_redeemer_record(redeemer))?;
        }

        for datum in tx.plutus_data() {
            self.append_from(self.to_plutus_datum_record(datum))?;
        }

        Ok(())
    }

    fn crawl_transaction(&mut self, tx: &MultiEraTx) -> Result<(), Error> {
        let record = self.to_transaction_record(tx);
        self.append_from(record.clone())?;

        // crawl inputs
        for (idx, input) in tx.inputs().iter().enumerate() {
            let mut child = self.child_writer(EventContext {
                input_idx: Some(idx),
                ..EventContext::default()
            });

            child.append_from(self.to_transaction_input_record(input))?;
        }

        for (idx, output) in tx.outputs().iter().enumerate() {
            let mut child = self.child_writer(EventContext {
                output_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_transaction_output(output)?;
        }

        //crawl certs
        for (idx, cert) in tx.certs().iter().enumerate() {
            if let Some(evt) = self.to_certificate_event(cert) {
                let mut child = self.child_writer(EventContext {
                    certificate_idx: Some(idx),
                    ..EventContext::default()
                });

                child.append(evt)?;
            }
        }

        for collateral in tx.collateral().iter() {
            // TODO: collateral context?
            self.crawl_collateral(collateral)?;
        }

        // crawl mints
        for asset in tx.mints() {
            self.append_from(self.to_mint_record(&asset))?;
        }

        self.crawl_metadata(tx)?;

        // crawl aux native scripts
        for script in tx.aux_native_scripts() {
            self.append(self.to_aux_native_script_event(script))?;
        }

        // crawl aux plutus v1 scripts
        for script in tx.aux_plutus_v1_scripts() {
            self.append(self.to_aux_plutus_script_event(script))?;
        }

        self.crawl_witnesses(tx)?;

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    fn crawl_block(&mut self, block: &MultiEraBlock, cbor: &[u8]) -> Result<(), Error> {
        let record = self.to_block_record(block, cbor);
        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.txs().iter().enumerate() {
            let mut child = self.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: Some(tx.hash().to_string()),
                ..EventContext::default()
            });

            child.crawl_transaction(tx)?;
        }

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    /// Mapper entry-point for raw cbor blocks
    pub fn crawl_cbor(&mut self, cbor: &[u8]) -> Result<(), Error> {
        let block = pallas::ledger::traverse::MultiEraBlock::decode(cbor)
            .map_err(OuraError::parse)
            .apply_policy(self.error_policy)
            .unwrap();

        if let Some(block) = block {
            let hash = block.hash();

            let mut child = self.child_writer(EventContext {
                block_hash: Some(hex::encode(hash)),
                block_number: Some(block.number()),
                slot: Some(block.slot()),
                timestamp: Some(block.wallclock(self.genesis)),
                ..EventContext::default()
            });

            child.crawl_block(&block, cbor)?;
        }

        Ok(())
    }

    pub fn crawl_rollback(&mut self, point: Point) -> Result<(), Error> {
        self.append(point.into())
    }
}
