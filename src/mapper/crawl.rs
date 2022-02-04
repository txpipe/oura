use pallas::ledger::alonzo::{
    crypto, AuxiliaryData, Block, Certificate, Metadata, Metadatum, Multiasset, TransactionBody,
    TransactionBodyComponent, TransactionInput, TransactionOutput, Value,
};

use pallas::crypto::hash::Hash;

use crate::{
    model::{EventContext, EventData, MetadataRecord, MetadatumRendition},
    Error,
};

use super::{map::ToHex, EventWriter};

impl EventWriter {
    fn crawl_metadata(&self, metadata: &Metadata) -> Result<(), Error> {
        for (label, content) in metadata.iter() {
            let record = self.to_metadata_record(label, content)?;
            self.append_from(record)?;

            // CIP25 block 6768751

            println!("---");
            println!("{:?}", metadata);

            let label: Result<_, Error> = match label {
                &Metadatum::Int(x) => Ok(x),
                // Text is sometimes seen instead of Int
                Metadatum::Text(x) => x.parse().map_err(|_| "Bad metadata label.".into()),
                _ => Err("Bad metadata label type.".into()),
            };

            // See:
            //
            //   * https://github.com/cardano-foundation/CIPs/blob/master/CIP-0010/registry.json
            //   * https://github.com/cardano-foundation/CIPs
            //
            match label? {
                // 674 => self.crawl_metadata_cip20(content),
                721 => self.crawl_metadata_label_721(content),
                61284 => self.crawl_metadata_cip15(content),
                _ => self.append(EventData::Metadata(MetadataRecord {
                    label: "TEST".to_string(),
                    content: MetadatumRendition::TextScalar(
                        "PARSE ERROR".to_string()
                    ),
                })),
            }?;
        }

        Ok(())
    }

    fn crawl_auxdata(&self, aux_data: &AuxiliaryData) -> Result<(), Error> {
        match aux_data {
            AuxiliaryData::Alonzo(data) => {
                if let Some(metadata) = &data.metadata {
                    self.crawl_metadata(metadata)?;
                }

                for _native in data.native_scripts.iter() {
                    self.append(self.to_native_script_event())?;
                }

                if let Some(plutus) = &data.plutus_scripts {
                    for script in plutus.iter() {
                        self.append(self.to_plutus_script_event(script))?;
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

                for _native in auxiliary_scripts.iter() {
                    self.append(self.to_native_script_event())?;
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
                    self.append_from(
                        self.to_transaction_output_asset_record(policy, asset, *amount),
                    )?;
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

    fn crawl_transaction(
        &self,
        tx: &TransactionBody,
        tx_hash: &str,
        aux_data: Option<&AuxiliaryData>,
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

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    fn crawl_block(&self, block: &Block, hash: &Hash<32>) -> Result<(), Error> {
        let record = self.to_block_record(block, hash)?;

        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.transaction_bodies.iter().enumerate() {
            let aux_data = block
                .auxiliary_data_set
                .iter()
                .find(|(k, _)| *k == (idx as u32))
                .map(|(_, v)| v);

            let tx_hash = crypto::hash_transaction(tx).to_hex();

            let child = self.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: Some(tx_hash.to_owned()),
                ..EventContext::default()
            });

            child.crawl_transaction(tx, &tx_hash, aux_data)?;
        }

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    pub fn crawl(&self, block: &Block) -> Result<(), Error> {
        let hash = crypto::hash_block_header(&block.header);

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(&hash)),
            block_number: Some(block.header.header_body.block_number),
            slot: Some(block.header.header_body.slot),
            timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_block(block, &hash)
    }
}
