use pallas::codec::utils::KeepRaw;
use pallas::ledger::primitives::ToHash;

use pallas::ledger::primitives::alonzo::{
    AuxiliaryData, Certificate, Metadata, MintedBlock, Multiasset, TransactionBody,
    TransactionInput, TransactionOutput, TransactionWitnessSet, Value,
};

use pallas::crypto::hash::Hash;

use crate::{
    model::{Era, EventContext, EventData},
    Error,
};

use super::{map::ToHex, EventWriter};

impl EventWriter {
    pub(crate) fn crawl_metadata(&self, metadata: &Metadata) -> Result<(), Error> {
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

    pub(crate) fn crawl_auxdata(&self, aux_data: &AuxiliaryData) -> Result<(), Error> {
        match aux_data {
            AuxiliaryData::PostAlonzo(data) => {
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
            AuxiliaryData::ShelleyMa(data) => {
                self.crawl_metadata(&data.transaction_metadata)?;

                if let Some(native) = &data.auxiliary_scripts {
                    for script in native.iter() {
                        self.append(self.to_aux_native_script_event(script))?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn crawl_transaction_input(&self, input: &TransactionInput) -> Result<(), Error> {
        self.append_from(self.to_transaction_input_record(input))
    }

    pub(crate) fn crawl_transaction_output_amount(&self, amount: &Value) -> Result<(), Error> {
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

    pub(crate) fn crawl_legacy_output(&self, output: &TransactionOutput) -> Result<(), Error> {
        let record = self.to_legacy_output_record(output)?;
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

    pub(crate) fn crawl_certificate(&self, certificate: &Certificate) -> Result<(), Error> {
        self.append(self.to_certificate_event(certificate))

        // more complex event goes here (eg: pool metadata?)
    }

    pub(crate) fn crawl_collateral(&self, collateral: &TransactionInput) -> Result<(), Error> {
        self.append(self.to_collateral_event(collateral))

        // TODO: should we have a collateral idx in context?
        // more complex event goes here (eg: ???)
    }

    pub(crate) fn crawl_mints(&self, mints: &Multiasset<i64>) -> Result<(), Error> {
        // should we have a policy context?

        for (policy, assets) in mints.iter() {
            for (asset, quantity) in assets.iter() {
                self.append_from(self.to_mint_record(policy, asset, *quantity))?;
            }
        }

        Ok(())
    }

    pub(crate) fn crawl_witness_set(
        &self,
        witness_set: &TransactionWitnessSet,
    ) -> Result<(), Error> {
        if let Some(native) = &witness_set.native_script {
            for script in native.iter() {
                self.append_from(self.to_native_witness_record(script)?)?;
            }
        }

        if let Some(plutus) = &witness_set.plutus_script {
            for script in plutus.iter() {
                self.append_from(self.to_plutus_v1_witness_record(script)?)?;
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
        tx: &KeepRaw<TransactionBody>,
        tx_hash: &str,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<TransactionWitnessSet>>,
    ) -> Result<(), Error> {
        let record = self.to_transaction_record(tx, tx_hash, aux_data, witness_set)?;

        self.append_from(record.clone())?;

        for (idx, input) in tx.inputs.iter().enumerate() {
            let child = self.child_writer(EventContext {
                input_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_transaction_input(input)?;
        }

        for (idx, output) in tx.outputs.iter().enumerate() {
            let child = self.child_writer(EventContext {
                output_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_legacy_output(output)?;
        }

        if let Some(certs) = &tx.certificates {
            for (idx, cert) in certs.iter().enumerate() {
                let child = self.child_writer(EventContext {
                    certificate_idx: Some(idx),
                    ..EventContext::default()
                });

                child.crawl_certificate(cert)?;
            }
        }

        if let Some(collateral) = &tx.collateral {
            for (_idx, collateral) in collateral.iter().enumerate() {
                // TODO: collateral context?

                self.crawl_collateral(collateral)?;
            }
        }

        if let Some(mint) = &tx.mint {
            self.crawl_mints(mint)?;
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
        block: &MintedBlock,
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
    pub fn crawl_with_cbor(&self, block: &MintedBlock, cbor: &[u8]) -> Result<(), Error> {
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
    pub fn crawl(&self, block: &MintedBlock) -> Result<(), Error> {
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
        block: &'b MintedBlock<'b>,
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
        let (_, block): (u16, MintedBlock) = pallas::codec::minicbor::decode(cbor)?;
        self.crawl_shelley_with_cbor(&block, cbor, era)
    }
}
