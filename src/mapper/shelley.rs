use pallas::ledger::primitives::{Fragment, ToHash};

use pallas::ledger::primitives::alonzo::{
    self, AuxiliaryData, Block, Certificate, Metadata, Multiasset, TransactionBody,
    TransactionBodyComponent, TransactionInput, TransactionOutput, TransactionWitnessSet, Value,
};

use pallas::crypto::hash::Hash;

use crate::{
    model::{Era, EventContext, EventData},
    Error,
};

use super::{map::ToHex, EventWriter};

impl EventWriter {
    fn crawl_metadata(&self, metadata: &Metadata) -> Result<(), Error> {
        for (label, content) in metadata.iter() {
            let record = self.to_metadata_record(label, content)?;
            self.append_from(record)?;

            match u64::from(label) {
                721u64 => self.crawl_metadata_label_721(content)?,
                61284u64 => self.crawl_metadata_label_61284(content)?,
                _ => (),
            }
        }

        Ok(())
    }

    fn crawl_auxdata(&self, aux_data: &AuxiliaryData) -> Result<(), Error> {
        match aux_data {
            AuxiliaryData::Alonzo(data) => {
                if let Some(metadata) = &data.metadata {
                    self.crawl_metadata(metadata)?;
                }

                if let Some(native) = &data.native_scripts {
                    for script in native.iter() {
                        self.append(self.to_aux_native_script_event(script))?;
                    }
                }

                if let Some(plutus) = &data.plutus_scripts {
                    for script in plutus.iter() {
                        self.append(self.to_aux_plutus_script_event(script))?;
                    }
                }
            }
            AuxiliaryData::Shelley(data) => {
                self.crawl_metadata(data)?;
            }
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                auxiliary_scripts,
            } => {
                self.crawl_metadata(transaction_metadata)?;

                if let Some(native) = &auxiliary_scripts {
                    for script in native.iter() {
                        self.append(self.to_aux_native_script_event(script))?;
                    }
                }
            }
        }

        Ok(())
    }

    fn crawl_transaction_input(&self, input: &TransactionInput) -> Result<(), Error> {
        self.append_from(self.to_transaction_input_record(input))
    }

    fn crawl_transaction_output_amount(&self, amount: &Value) -> Result<(), Error> {
        if let Value::Multiasset(_, policies) = amount {
            for (policy, assets) in policies.iter() {
                for (asset, amount) in assets.iter() {
                    self.append_from(self.to_transaction_output_asset_record(
                        policy,
                        asset,
                        amount.into(),
                    ))?;
                }
            }
        }

        Ok(())
    }

    fn crawl_transaction_output(&self, output: &TransactionOutput) -> Result<(), Error> {
        let record = self.to_transaction_output_record(output)?;
        self.append(record.into())?;

        let child = &self.child_writer(EventContext {
            output_address: self
                .utils
                .bech32
                .encode_address(output.address.as_slice())?
                .into(),
            ..EventContext::default()
        });

        child.crawl_transaction_output_amount(&output.amount)?;

        Ok(())
    }

    fn crawl_certificate(&self, certificate: &Certificate) -> Result<(), Error> {
        self.append(self.to_certificate_event(certificate))

        // more complex event goes here (eg: pool metadata?)
    }

    fn crawl_collateral(&self, collateral: &TransactionInput) -> Result<(), Error> {
        self.append(self.to_collateral_event(collateral))

        // TODO: should we have a collateral idx in context?
        // more complex event goes here (eg: ???)
    }

    fn crawl_mints(&self, mints: &Multiasset<i64>) -> Result<(), Error> {
        // should we have a policy context?

        for (policy, assets) in mints.iter() {
            for (asset, quantity) in assets.iter() {
                self.append_from(self.to_mint_record(policy, asset, *quantity))?;
            }
        }

        Ok(())
    }

    fn crawl_witness_set(&self, witness_set: &TransactionWitnessSet) -> Result<(), Error> {
        if let Some(native) = &witness_set.native_script {
            for script in native.iter() {
                self.append_from(self.to_native_witness_record(script)?)?;
            }
        }

        if let Some(plutus) = &witness_set.plutus_script {
            for script in plutus.iter() {
                self.append_from(self.to_plutus_witness_record(script)?)?;
            }
        }

        if let Some(redeemers) = &witness_set.redeemer {
            for redeemer in redeemers.iter() {
                self.append_from(self.to_plutus_redeemer_record(redeemer)?)?;
            }
        }

        if let Some(datums) = &witness_set.plutus_data {
            for datum in datums.iter() {
                self.append_from(self.to_plutus_datum_record(datum)?)?;
            }
        }

        Ok(())
    }

    fn crawl_shelley_transaction(
        &self,
        tx: &TransactionBody,
        tx_hash: &str,
        aux_data: Option<&AuxiliaryData>,
        witness_set: Option<&TransactionWitnessSet>,
    ) -> Result<(), Error> {
        let record = self.to_transaction_record(tx, tx_hash, aux_data)?;

        self.append_from(record.clone())?;

        for component in tx.iter() {
            match component {
                TransactionBodyComponent::Inputs(x) => {
                    for (idx, input) in x.iter().enumerate() {
                        let child = self.child_writer(EventContext {
                            input_idx: Some(idx),
                            ..EventContext::default()
                        });

                        child.crawl_transaction_input(input)?;
                    }
                }
                TransactionBodyComponent::Outputs(x) => {
                    for (idx, output) in x.iter().enumerate() {
                        let child = self.child_writer(EventContext {
                            output_idx: Some(idx),
                            ..EventContext::default()
                        });

                        child.crawl_transaction_output(output)?;
                    }
                }
                TransactionBodyComponent::Certificates(certs) => {
                    for (idx, cert) in certs.iter().enumerate() {
                        let child = self.child_writer(EventContext {
                            certificate_idx: Some(idx),
                            ..EventContext::default()
                        });

                        child.crawl_certificate(cert)?;
                    }
                }
                TransactionBodyComponent::Collateral(collaterals) => {
                    for (_idx, collateral) in collaterals.iter().enumerate() {
                        // TODO: collateral context?

                        self.crawl_collateral(collateral)?;
                    }
                }
                TransactionBodyComponent::Mint(x) => {
                    self.crawl_mints(x)?;
                }
                _ => (),
            };
        }

        if let Some(aux_data) = aux_data {
            self.crawl_auxdata(aux_data)?;
        }

        if let Some(witness_set) = witness_set {
            self.crawl_witness_set(witness_set)?;
        }

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    fn crawl_shelley_block(
        &self,
        block: &Block,
        hash: &Hash<32>,
        cbor: &[u8],
        era: Era,
    ) -> Result<(), Error> {
        let record = self.to_block_record(block, hash, cbor, era)?;

        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.transaction_bodies.iter().enumerate() {
            let aux_data = block
                .auxiliary_data_set
                .iter()
                .find(|(k, _)| *k == (idx as u32))
                .map(|(_, v)| v);

            let witness_set = block.transaction_witness_sets.get(idx);

            let tx_hash = tx.to_hash().to_hex();

            let child = self.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: Some(tx_hash.to_owned()),
                ..EventContext::default()
            });

            child.crawl_shelley_transaction(tx, &tx_hash, aux_data, witness_set)?;
        }

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    #[deprecated(note = "use crawl_from_shelley_cbor instead")]
    pub fn crawl_with_cbor(&self, block: &Block, cbor: &[u8]) -> Result<(), Error> {
        let hash = block.header.to_hash();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            block_number: Some(block.header.header_body.block_number),
            slot: Some(block.header.header_body.slot),
            timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_shelley_block(block, &hash, cbor, Era::Undefined)
    }

    #[deprecated(note = "use crawl_from_shelley_cbor instead")]
    pub fn crawl(&self, block: &Block) -> Result<(), Error> {
        let hash = block.header.to_hash();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            block_number: Some(block.header.header_body.block_number),
            slot: Some(block.header.header_body.slot),
            timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_shelley_block(block, &hash, &[], Era::Undefined)
    }

    /// Mapper entry-point for decoded Shelley blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we already have a decoded block (for example, N2C). The raw CBOR is also
    /// passed through in case we need to attach it to outbound events.
    pub fn crawl_shelley_with_cbor<'b>(
        &self,
        block: &'b Block,
        cbor: &'b [u8],
        era: Era,
    ) -> Result<(), Error> {
        let hash = block.header.to_hash();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            block_number: Some(block.header.header_body.block_number),
            slot: Some(block.header.header_body.slot),
            timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_shelley_block(block, &hash, cbor, era)
    }

    /// Mapper entry-point for raw Shelley cbor blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we haven't decoded the CBOR yet (for example, N2N).
    ///
    /// We use Alonzo primitives since they are backward compatible with
    /// Shelley. In this way, we can avoid having to fork the crawling procedure
    /// for each different hard-fork.
    pub fn crawl_from_shelley_cbor(&self, cbor: &[u8], era: Era) -> Result<(), Error> {
        let alonzo::BlockWrapper(_, block) = alonzo::BlockWrapper::decode_fragment(cbor)?;
        self.crawl_shelley_with_cbor(&block, cbor, era)
    }
}
