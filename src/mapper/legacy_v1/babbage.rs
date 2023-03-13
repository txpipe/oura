use pallas::codec::utils::KeepRaw;

use pallas::ledger::primitives::babbage::{
    AuxiliaryData, MintedBlock, MintedDatumOption, MintedTransactionBody, MintedTransactionOutput,
    MintedWitnessSet, NetworkId,
};

use pallas::ledger::traverse::{
    MultiEraAsset, MultiEraBlock, MultiEraCert, MultiEraInput, MultiEraOutput, MultiEraTx,
    OriginalHash,
};

use crate::framework::legacy_v1::*;
use crate::utils::time::TimeProvider;
use crate::Error;

use super::{map::ToHex, EventWriter};

impl EventWriter {
    pub fn collect_withdrawal_records(
        &self,
        withdrawls: &KeyValuePairs<RewardAccount, Coin>,
    ) -> Vec<WithdrawalRecord> {
        withdrawls
            .iter()
            .map(|(reward_account, coin)| WithdrawalRecord {
                reward_account: {
                    let hex = reward_account.to_hex();
                    hex.strip_prefix("e1").map(|x| x.to_string()).unwrap_or(hex)
                },
                coin: *coin,
            })
            .collect()
    }

    pub fn to_transaction_record(&self, tx: &MultiEraTx) -> Result<TransactionRecord, Error> {
        let mut record = TransactionRecord {
            hash: tx.hash().to_string(),
            size: tx.size() as u32,
            // TODO: we have a problem here. AFAIK, there's no reference to the tx fee in Byron
            // block content. This leaves us with the two alternative: a) compute the value, b)
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
            fee: tx.fee().unwrap_or_default(),
            ttl: tx.ttl(),
            validity_interval_start: body.validity_interval_start,
            network_id: body.network_id.as_ref().map(|x| match x {
                NetworkId::One => 1,
                NetworkId::Two => 2,
            }),
            ..Default::default()
        };

        let outputs: Vec<_> = tx
            .outputs()
            .iter()
            .map(|x| self.to_transaction_output_record(x))
            .collect()?;

        record.output_count = outputs.len();
        record.total_output = outputs.iter().map(|o| o.amount).sum();

        let inputs: Vec<_> = tx
            .inputs()
            .iter()
            .map(|x| self.to_transaction_input_record(x))
            .collect()?;

        record.input_count = inputs.len();

        let mints: Vec<_> = tx
            .mints()
            .iter()
            .map(|x| self.to_mint_record(x))
            .collect()?;

        record.mint_count = mints.len();

        let collateral_inputs: Vec<_> = tx
            .collateral()
            .iter()
            .map(|x| self.to_transaction_input_record(x))
            .collect()?;

        record.collateral_input_count = collateral_inputs.len();
        record.has_collateral_output = body.collateral_return.is_some();

        // TODO
        // TransactionBodyComponent::ScriptDataHash(_)
        // TransactionBodyComponent::RequiredSigners(_)
        // TransactionBodyComponent::AuxiliaryDataHash(_)

        if self.config.include_transaction_details {
            record.outputs = Some(outputs);
            record.inputs = Some(inputs);
            record.mint = Some(mints);

            record.collateral_inputs = Some(collateral_inputs);

            record.collateral_output = body.collateral_return.as_ref().map(|output| match output {
                MintedTransactionOutput::Legacy(x) => self.to_legacy_output_record(x).unwrap(),
                MintedTransactionOutput::PostAlonzo(x) => {
                    self.to_post_alonzo_output_record(x).unwrap()
                }
            });

            record.metadata = match aux_data {
                Some(aux_data) => self.collect_metadata_records(aux_data)?.into(),
                None => None,
            };

            record.vkey_witnesses = tx
                .vkey_witnesses()
                .iter()
                .map(|x| self.to_vkey_witness_record(x))
                .collect()?
                .into();

            record.native_witnesses = tx
                .native_scripts()
                .iter()
                .map(|x| self.to_native_witness_record(x))
                .collect()?
                .into();

            let v1_scripts = tx
                .plutus_v1_scripts()
                .iter()
                .map(|x| self.to_plutus_v1_witness_record(x))
                .collect()?
                .into();

            let v2_scripts = tx
                .plutus_v2_scripts()
                .iter()
                .map(|x| self.to_plutus_v2_witness_record(x))
                .collect()?
                .into();

            record.plutus_witnesses = [v1_scripts, v2_scripts].concat();

            record.plutus_redeemers = tx
                .redeemers()
                .iter()
                .map(|x| self.to_plutus_redeemer_record(x))
                .collect()?
                .into();

            record.plutus_data = tx
                .plutus_data()
                .iter()
                .map(|x| self.to_plutus_datum_record(x))
                .collect()?
                .into();

            if let Some(withdrawals) = &body.withdrawals {
                record.withdrawals = self.collect_withdrawal_records(withdrawals).into();
            }
        }

        Ok(record)
    }

    pub fn to_block_record(
        &self,
        source: &MultiEraBlock,
        cbor: &[u8],
    ) -> Result<BlockRecord, Error> {
        let relative_epoch = self
            .utils
            .time
            .as_ref()
            .map(|time| time.absolute_slot_to_relative(source.slot()));

        let mut record = BlockRecord {
            era: source.era(),
            body_size: source.body_size(),
            issuer_vkey: source.source.header.header_body.issuer_vkey.to_hex(),
            vrf_vkey: source.header.header_body.vrf_vkey.to_hex(),
            tx_count: source.tx_count(),
            hash: source.hash().to_string(),
            number: source.number(),
            slot: source.slot(),
            epoch: relative_epoch.map(|(epoch, _)| epoch),
            epoch_slot: relative_epoch.map(|(_, epoch_slot)| epoch_slot),
            previous_hash: source
                .header
                .header_body
                .prev_hash
                .map(hex::encode)
                .unwrap_or_default(),
            cbor_hex: match self.config.include_block_cbor {
                true => source.encode(),
                false => None,
            },
            transactions: None,
        };

        if self.config.include_block_details {
            let txs: Vec<_> = source
                .txs()
                .iter()
                .map(|x| self.to_transaction_record(x))
                .collect()?;

            record.transactions = Some(txs);
        }

        Ok(record)
    }

    fn crawl_certificate(&self, certificate: &MultiEraCert) -> Result<(), Error> {
        self.append(self.to_certificate_event(certificate))

        // more complex event goes here (eg: pool metadata?)
    }

    fn crawl_collateral(&self, collateral: &MultiEraInput) -> Result<(), Error> {
        self.append(self.to_collateral_event(collateral))

        // TODO: should we have a collateral idx in context?
        // more complex event goes here (eg: ???)
    }

    pub fn to_mint_record(&self, asset: &MultiEraAsset) -> MintRecord {
        MintRecord {
            policy: asset.policy().map(|x| x.to_string()).unwrap_or_default(),
            asset: asset.name().map(|x| hex::encode(x)).unwrap_or_default(),
            quantity: asset.coin(),
        }
    }

    fn crawl_mints(&self, tx: &MultiEraTx) -> Result<(), Error> {
        // should we have a policy context?

        for asset in tx.mints() {
            self.append_from(self.to_mint_record(&asset))?;
        }

        Ok(())
    }

    fn crawl_metadata(&self, tx: &MultiEraTx) -> Result<(), Error> {
        let metadata = tx.metadata().collect::<Vec<_>>();

        for (label, content) in metadata.iter() {
            let record = self.to_metadata_record(label, content)?;
            self.append_from(record)?;

            match label {
                721u64 => self.crawl_metadata_label_721(content)?,
                61284u64 => self.crawl_metadata_label_61284(content)?,
                _ => (),
            }
        }

        Ok(())
    }

    fn crawl_auxdata(&self, tx: &MultiEraTx) -> Result<(), Error> {
        self.crawl_metadata(tx);

        for script in tx.aux_native_scripts() {
            self.append(self.to_aux_native_script_event(script))?;
        }

        for script in tx.aux_plutus_v1_scripts() {
            self.append(self.to_aux_plutus_script_event(script))?;
        }

        Ok(())
    }

    pub fn to_transaction_input_record(&self, input: &MultiEraInput) -> TxInputRecord {
        TxInputRecord {
            tx_id: input.hash().to_string(),
            index: input.index(),
        }
    }

    pub fn to_transaction_output_asset_record(&self, asset: &MultiEraAsset) -> OutputAssetRecord {
        OutputAssetRecord {
            policy: asset.policy().map(ToString::to_string).unwrap_or_default(),
            asset: asset.name().map(|x| x.to_hex()).unwrap_or_default(),
            asset_ascii: asset.to_ascii_name(),
            amount: asset.coin() as u64,
        }
    }

    pub fn to_transaction_output_record(
        &self,
        output: &MultiEraOutput,
    ) -> Result<TxOutputRecord, Error> {
        let address = output.address().map_err(Error::parse)?;

        Ok(TxOutputRecord {
            address: address.to_string(),
            amount: output.lovelace_amount(),
            assets: output
                .non_ada_assets()
                .iter()
                .map(|x| self.to_transaction_output_asset_record(x))
                .collect()
                .into(),
            datum_hash: match &output.datum() {
                Some(MintedDatumOption::Hash(x)) => Some(x.to_string()),
                Some(MintedDatumOption::Data(x)) => Some(x.original_hash().to_hex()),
                None => None,
            },
            inline_datum: match &output.datum() {
                Some(MintedDatumOption::Data(x)) => Some(self.to_plutus_datum_record(x)?),
                _ => None,
            },
        })
    }

    fn crawl_transaction_output(&self, output: &MultiEraOutput) -> Result<(), Error> {
        let record = self.to_transaction_output_record(output)?;
        self.append(record.into())?;

        let address = output.address().map_err(Error::parse)?;

        let child = &self.child_writer(EventContext {
            output_address: address.to_string().into(),
            ..EventContext::default()
        });

        for asset in output.assets() {
            self.append_from(self.to_transaction_output_asset_record(&asset))?;
        }

        if let Some(MintedDatumOption::Data(datum)) = &output.datum() {
            let record = self.to_plutus_datum_record(datum)?;
            child.append(record.into())?;
        }

        Ok(())
    }

    fn crawl_witnesses(&self, tx: &MultiEraTx) -> Result<(), Error> {
        for script in tx.native_scripts() {
            self.append_from(self.to_native_witness_record(script)?)?;
        }

        for script in tx.plutus_v1_scripts() {
            self.append_from(self.to_plutus_v1_witness_record(script)?)?;
        }

        for script in tx.plutus_v2_scripts() {
            self.append_from(self.to_plutus_v2_witness_record(script)?)?;
        }

        for redeemer in tx.redeemers() {
            self.append_from(self.to_plutus_redeemer_record(redeemer)?)?;
        }

        for datum in tx.plutus_data() {
            self.append_from(self.to_plutus_datum_record(datum)?)?;
        }

        Ok(())
    }

    fn crawl_transaction(&self, tx: &MultiEraTx) -> Result<(), Error> {
        let record = self.to_transaction_record(tx)?;
        self.append_from(record.clone())?;

        for (idx, input) in tx.inputs().iter().enumerate() {
            let child = self.child_writer(EventContext {
                input_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_transaction_input(input)?;
        }

        for (idx, output) in tx.outputs().iter().enumerate() {
            let child = self.child_writer(EventContext {
                output_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_transaction_output(output)?;
        }

        for (idx, cert) in tx.certs().iter().enumerate() {
            let child = self.child_writer(EventContext {
                certificate_idx: Some(idx),
                ..EventContext::default()
            });

            child.crawl_certificate(cert)?;
        }

        for collateral in tx.collateral().iter() {
            // TODO: collateral context?
            self.crawl_collateral(collateral)?;
        }

        self.crawl_mints(tx)?;

        self.crawl_auxdata(tx)?;

        self.crawl_witnesses(tx)?;

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    fn crawl_block(&self, block: &MultiEraBlock, cbor: &[u8]) -> Result<(), Error> {
        let record = self.to_block_record(block, cbor)?;
        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.txs().iter().enumerate() {
            let child = self.child_writer(EventContext {
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
    pub fn crawl_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let block = pallas::ledger::traverse::MultiEraBlock::decode(cbor).map_err(Error::parse)?;

        let hash = block.hash();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(hash)),
            block_number: Some(block.number()),
            slot: Some(block.slot()),
            timestamp: self.compute_timestamp(block.slot()),
            ..EventContext::default()
        });

        child.crawl_block(&block, cbor);

        Ok(())
    }
}
