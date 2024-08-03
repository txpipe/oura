use std::ops::Deref;

use pallas_codec::utils::{KeepRaw, NonZeroInt};

use pallas_primitives::conway::{
    AuxiliaryData, Certificate, MintedBlock, MintedDatumOption, MintedPostAlonzoTransactionOutput,
    MintedTransactionBody, MintedTransactionOutput, MintedWitnessSet, Multiasset, NetworkId,
    RedeemerTag, RedeemersKey, RedeemersValue,
};

use pallas_crypto::hash::Hash;
use pallas_primitives::ToCanonicalJson as _;
use pallas_traverse::OriginalHash;

use crate::model::{
    BlockRecord, Era, MintRecord, PlutusRedeemerRecord, TransactionRecord, TxOutputRecord,
};
use crate::utils::time::TimeProvider;
use crate::{
    model::{EventContext, EventData},
    Error,
};

use super::{map::ToHex, EventWriter};

impl EventWriter {
    pub fn collect_conway_mint_records(&self, mint: &Multiasset<NonZeroInt>) -> Vec<MintRecord> {
        mint.iter()
            .flat_map(|(policy, assets)| {
                assets
                    .iter()
                    .map(|(asset, amount)| self.to_mint_record(policy, asset, amount.into()))
            })
            .collect()
    }

    pub fn crawl_conway_mints(&self, mints: &Multiasset<NonZeroInt>) -> Result<(), Error> {
        for (policy, assets) in mints.iter() {
            for (asset, quantity) in assets.iter() {
                self.append_from(self.to_mint_record(policy, asset, quantity.into()))?;
            }
        }

        Ok(())
    }

    pub fn to_conway_output_record(
        &self,
        output: &MintedPostAlonzoTransactionOutput,
    ) -> Result<TxOutputRecord, Error> {
        let address = pallas_addresses::Address::from_bytes(&output.address)?;

        Ok(TxOutputRecord {
            address: address.to_string(),
            amount: super::map::get_tx_output_coin_value(&output.value),
            assets: self.collect_asset_records(&output.value).into(),
            datum_hash: match &output.datum_option {
                Some(MintedDatumOption::Hash(x)) => Some(x.to_string()),
                Some(MintedDatumOption::Data(x)) => Some(x.original_hash().to_hex()),
                None => None,
            },
            inline_datum: match &output.datum_option {
                Some(MintedDatumOption::Data(x)) => Some(self.to_plutus_datum_record(x)?),
                _ => None,
            },
        })
    }

    pub fn to_conway_redeemer_record(
        &self,
        key: &RedeemersKey,
        value: &RedeemersValue,
    ) -> Result<PlutusRedeemerRecord, crate::Error> {
        Ok(PlutusRedeemerRecord {
            purpose: match key.tag {
                RedeemerTag::Spend => "spend".to_string(),
                RedeemerTag::Mint => "mint".to_string(),
                RedeemerTag::Cert => "cert".to_string(),
                RedeemerTag::Reward => "reward".to_string(),
                RedeemerTag::Vote => "vote".to_string(),
                RedeemerTag::Propose => "propose".to_string(),
            },
            ex_units_mem: value.ex_units.mem,
            ex_units_steps: value.ex_units.steps,
            input_idx: key.index,
            plutus_data: value.data.to_json(),
        })
    }

    pub fn collect_conway_output_records(
        &self,
        source: &[MintedTransactionOutput],
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .iter()
            .map(|x| match x {
                MintedTransactionOutput::Legacy(x) => self.to_legacy_output_record(x),
                MintedTransactionOutput::PostAlonzo(x) => self.to_conway_output_record(x),
            })
            .collect()
    }

    pub fn to_conway_tx_size(
        &self,
        body: &KeepRaw<MintedTransactionBody>,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<MintedWitnessSet>>,
    ) -> usize {
        body.raw_cbor().len()
            + aux_data.map(|ax| ax.raw_cbor().len()).unwrap_or(2)
            + witness_set.map(|ws| ws.raw_cbor().len()).unwrap_or(1)
    }

    pub fn to_conway_transaction_record(
        &self,
        body: &KeepRaw<MintedTransactionBody>,
        tx_hash: &str,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<MintedWitnessSet>>,
    ) -> Result<TransactionRecord, Error> {
        let mut record = TransactionRecord {
            hash: tx_hash.to_owned(),
            size: self.to_conway_tx_size(body, aux_data, witness_set) as u32,
            fee: body.fee,
            ttl: body.ttl,
            validity_interval_start: body.validity_interval_start,
            network_id: body.network_id.as_ref().map(|x| match x {
                NetworkId::One => 1,
                NetworkId::Two => 2,
            }),
            ..Default::default()
        };

        let outputs = self.collect_conway_output_records(body.outputs.as_slice())?;
        record.output_count = outputs.len();
        record.total_output = outputs.iter().map(|o| o.amount).sum();

        let inputs = self.collect_input_records(&body.inputs);
        record.input_count = inputs.len();

        if let Some(mint) = &body.mint {
            let mints = self.collect_conway_mint_records(mint);
            record.mint_count = mints.len();

            if self.config.include_transaction_details {
                record.mint = mints.into();
            }
        }

        // Add Collateral Stuff
        let collateral_inputs = &body.collateral.as_deref();
        record.collateral_input_count = collateral_inputs.iter().count();
        record.has_collateral_output = body.collateral_return.is_some();

        // TODO
        // TransactionBodyComponent::ScriptDataHash(_)
        // TransactionBodyComponent::RequiredSigners(_)
        // TransactionBodyComponent::AuxiliaryDataHash(_)

        if self.config.include_transaction_details {
            record.outputs = outputs.into();
            record.inputs = inputs.into();

            // transaction_details collateral stuff
            record.collateral_inputs =
                collateral_inputs.map(|inputs| self.collect_input_records(inputs));

            record.collateral_output = body.collateral_return.as_ref().map(|output| match output {
                MintedTransactionOutput::Legacy(x) => self.to_legacy_output_record(x).unwrap(),
                MintedTransactionOutput::PostAlonzo(x) => self.to_conway_output_record(x).unwrap(),
            });

            record.metadata = match aux_data {
                Some(aux_data) => self.collect_metadata_records(aux_data)?.into(),
                None => None,
            };

            if let Some(witnesses) = witness_set {
                record.vkey_witnesses = Some(
                    witnesses
                        .vkeywitness
                        .iter()
                        .flatten()
                        .map(|i| self.to_vkey_witness_record(i))
                        .collect::<Result<_, _>>()?,
                );

                record.native_witnesses = Some(
                    witnesses
                        .native_script
                        .iter()
                        .flatten()
                        .map(|i| self.to_native_witness_record(i))
                        .collect::<Result<_, _>>()?,
                );

                let mut all_plutus = vec![];

                let plutus_v1: Vec<_> = witnesses
                    .plutus_v1_script
                    .iter()
                    .flatten()
                    .map(|i| self.to_plutus_v1_witness_record(i))
                    .collect::<Result<_, _>>()?;

                all_plutus.extend(plutus_v1);

                let plutus_v2: Vec<_> = witnesses
                    .plutus_v2_script
                    .iter()
                    .flatten()
                    .map(|i| self.to_plutus_v2_witness_record(i))
                    .collect::<Result<_, _>>()?;

                all_plutus.extend(plutus_v2);

                let plutus_v3: Vec<_> = witnesses
                    .plutus_v3_script
                    .iter()
                    .flatten()
                    .map(|i| self.to_plutus_v3_witness_record(i))
                    .collect::<Result<_, _>>()?;

                all_plutus.extend(plutus_v3);

                record.plutus_witnesses = Some(all_plutus);

                record.plutus_redeemers = Some(
                    witnesses
                        .redeemer
                        .iter()
                        .map(|i| i.iter())
                        .flatten()
                        .map(|(k, v)| self.to_conway_redeemer_record(&k, &v))
                        .collect::<Result<_, _>>()?,
                );

                record.plutus_data = Some(
                    witnesses
                        .plutus_data
                        .iter()
                        .flatten()
                        .map(|i| self.to_plutus_datum_record(i))
                        .collect::<Result<_, _>>()?,
                );
            }

            if let Some(withdrawals) = &body.withdrawals {
                record.withdrawals = self.collect_withdrawal_records(withdrawals).into();
            }
        }

        Ok(record)
    }

    pub fn to_conway_block_record(
        &self,
        source: &MintedBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<BlockRecord, Error> {
        let relative_epoch = self
            .utils
            .time
            .as_ref()
            .map(|time| time.absolute_slot_to_relative(source.header.header_body.slot));

        let mut record = BlockRecord {
            era: Era::Conway,
            body_size: source.header.header_body.block_body_size as usize,
            issuer_vkey: source.header.header_body.issuer_vkey.to_hex(),
            vrf_vkey: source.header.header_body.vrf_vkey.to_hex(),
            tx_count: source.transaction_bodies.len(),
            hash: hex::encode(hash),
            number: source.header.header_body.block_number,
            slot: source.header.header_body.slot,
            epoch: relative_epoch.map(|(epoch, _)| epoch),
            epoch_slot: relative_epoch.map(|(_, epoch_slot)| epoch_slot),
            previous_hash: source
                .header
                .header_body
                .prev_hash
                .map(hex::encode)
                .unwrap_or_default(),
            cbor_hex: match self.config.include_block_cbor {
                true => hex::encode(cbor).into(),
                false => None,
            },
            transactions: None,
        };

        if self.config.include_block_details || self.config.include_transaction_details {
            record.transactions = Some(self.collect_conway_tx_records(source)?);
        }

        Ok(record)
    }

    pub fn collect_conway_tx_records(
        &self,
        block: &MintedBlock,
    ) -> Result<Vec<TransactionRecord>, Error> {
        block
            .transaction_bodies
            .iter()
            .enumerate()
            .map(|(idx, tx)| {
                let aux_data = block
                    .auxiliary_data_set
                    .iter()
                    .find(|(k, _)| *k == (idx as u32))
                    .map(|(_, v)| v);

                let witness_set = block.transaction_witness_sets.get(idx);

                let tx_hash = tx.original_hash().to_hex();

                self.to_conway_transaction_record(tx, &tx_hash, aux_data, witness_set)
            })
            .collect()
    }

    fn crawl_conway_output(&self, output: &MintedPostAlonzoTransactionOutput) -> Result<(), Error> {
        let record = self.to_conway_output_record(output)?;
        self.append(record.into())?;

        let address = pallas_addresses::Address::from_bytes(&output.address)?;

        let child = &self.child_writer(EventContext {
            output_address: address.to_string().into(),
            ..EventContext::default()
        });

        child.crawl_transaction_output_amount(&output.value)?;

        if let Some(MintedDatumOption::Data(datum)) = &output.datum_option {
            let record = self.to_plutus_datum_record(datum)?;
            child.append(record.into())?;
        }

        Ok(())
    }

    fn crawl_conway_transaction_output(
        &self,
        output: &MintedTransactionOutput,
    ) -> Result<(), Error> {
        match output {
            MintedTransactionOutput::Legacy(x) => self.crawl_legacy_output(x),
            MintedTransactionOutput::PostAlonzo(x) => self.crawl_conway_output(x),
        }
    }

    fn crawl_conway_witness_set(
        &self,
        witness_set: &KeepRaw<MintedWitnessSet>,
    ) -> Result<(), Error> {
        if let Some(native) = &witness_set.native_script {
            for script in native.iter() {
                self.append_from(self.to_native_witness_record(script)?)?;
            }
        }

        if let Some(plutus) = &witness_set.plutus_v1_script {
            for script in plutus.iter() {
                self.append_from(self.to_plutus_v1_witness_record(script)?)?;
            }
        }

        if let Some(plutus) = &witness_set.plutus_v2_script {
            for script in plutus.iter() {
                self.append_from(self.to_plutus_v2_witness_record(script)?)?;
            }
        }

        if let Some(plutus) = &witness_set.plutus_v3_script {
            for script in plutus.iter() {
                self.append_from(self.to_plutus_v3_witness_record(script)?)?;
            }
        }

        if let Some(redeemers) = &witness_set.redeemer {
            for (key, value) in redeemers.iter() {
                self.append_from(self.to_conway_redeemer_record(key, value)?)?;
            }
        }

        if let Some(datums) = &witness_set.plutus_data {
            for datum in datums.iter() {
                self.append_from(self.to_plutus_datum_record(datum)?)?;
            }
        }

        Ok(())
    }

    pub fn to_conway_certificate_event(&self, certificate: &Certificate) -> Option<EventData> {
        match certificate {
            Certificate::StakeRegistration(credential) => EventData::StakeRegistration {
                credential: credential.into(),
            }
            .into(),
            Certificate::StakeDeregistration(credential) => EventData::StakeDeregistration {
                credential: credential.into(),
            }
            .into(),
            Certificate::StakeDelegation(credential, pool) => EventData::StakeDelegation {
                credential: credential.into(),
                pool_hash: pool.to_hex(),
            }
            .into(),
            Certificate::PoolRegistration {
                operator,
                vrf_keyhash,
                pledge,
                cost,
                margin,
                reward_account,
                pool_owners,
                relays,
                pool_metadata,
            } => EventData::PoolRegistration {
                operator: operator.to_hex(),
                vrf_keyhash: vrf_keyhash.to_hex(),
                pledge: *pledge,
                cost: *cost,
                margin: (margin.numerator as f64 / margin.denominator as f64),
                reward_account: reward_account.to_hex(),
                pool_owners: pool_owners.iter().map(|p| p.to_hex()).collect(),
                relays: relays.iter().map(super::map::relay_to_string).collect(),
                pool_metadata: pool_metadata.to_owned().map(|m| m.url.clone()).into(),
                pool_metadata_hash: pool_metadata
                    .to_owned()
                    .map(|m| m.hash.clone().to_hex())
                    .into(),
            }
            .into(),
            Certificate::PoolRetirement(pool, epoch) => EventData::PoolRetirement {
                pool: pool.to_hex(),
                epoch: *epoch,
            }
            .into(),
            // all new Conway certs are out of scope for Oura lts/v1
            _ => None,
        }
    }

    fn crawl_conway_certificate(&self, certificate: &Certificate) -> Result<(), Error> {
        if let Some(evt) = self.to_conway_certificate_event(certificate) {
            self.append(evt)?;
        }

        Ok(())
    }

    fn crawl_conway_transaction(
        &self,
        tx: &KeepRaw<MintedTransactionBody>,
        tx_hash: &str,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<MintedWitnessSet>>,
    ) -> Result<(), Error> {
        let record = self.to_conway_transaction_record(tx, tx_hash, aux_data, witness_set)?;

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

            child.crawl_conway_transaction_output(output)?;
        }

        if let Some(certs) = &tx.certificates {
            for (idx, cert) in certs.iter().enumerate() {
                let child = self.child_writer(EventContext {
                    certificate_idx: Some(idx),
                    ..EventContext::default()
                });

                child.crawl_conway_certificate(cert)?;
            }
        }

        if let Some(collateral) = &tx.collateral {
            for collateral in collateral.iter() {
                // TODO: collateral context?

                self.crawl_collateral(collateral)?;
            }
        }

        if let Some(mint) = &tx.mint {
            self.crawl_conway_mints(mint)?;
        }

        if let Some(aux_data) = aux_data {
            self.crawl_auxdata(aux_data)?;
        }

        if let Some(witness_set) = witness_set {
            self.crawl_conway_witness_set(witness_set)?;
        }

        if self.config.include_transaction_end_events {
            self.append(EventData::TransactionEnd(record))?;
        }

        Ok(())
    }

    fn crawl_conway_block(
        &self,
        block: &MintedBlock,
        hash: &Hash<32>,
        cbor: &[u8],
    ) -> Result<(), Error> {
        let record = self.to_conway_block_record(block, hash, cbor)?;

        self.append(EventData::Block(record.clone()))?;

        for (idx, tx) in block.transaction_bodies.iter().enumerate() {
            let aux_data = block
                .auxiliary_data_set
                .iter()
                .find(|(k, _)| *k == (idx as u32))
                .map(|(_, v)| v);

            let witness_set = block.transaction_witness_sets.get(idx);

            let tx_hash = tx.original_hash().to_hex();

            let child = self.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: Some(tx_hash.to_owned()),
                ..EventContext::default()
            });

            child.crawl_conway_transaction(tx, &tx_hash, aux_data, witness_set)?;
        }

        if self.config.include_block_end_events {
            self.append(EventData::BlockEnd(record))?;
        }

        Ok(())
    }

    /// Mapper entry-point for decoded Conway blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we already have a decoded block (for example, N2C). The raw CBOR is also
    /// passed through in case we need to attach it to outbound events.
    pub fn crawl_conway_with_cbor<'b>(
        &self,
        block: &'b MintedBlock<'b>,
        cbor: &'b [u8],
    ) -> Result<(), Error> {
        let hash = block.header.original_hash();

        let child = self.child_writer(EventContext {
            block_hash: Some(hex::encode(hash)),
            block_number: Some(block.header.header_body.block_number),
            slot: Some(block.header.header_body.slot),
            timestamp: self.compute_timestamp(block.header.header_body.slot),
            ..EventContext::default()
        });

        child.crawl_conway_block(block, &hash, cbor)
    }

    /// Mapper entry-point for raw Conway cbor blocks
    ///
    /// Entry-point to start crawling a blocks for events. Meant to be used when
    /// we haven't decoded the CBOR yet (for example, N2N).
    pub fn crawl_from_conway_cbor(&self, cbor: &[u8]) -> Result<(), Error> {
        let (_, block): (u16, MintedBlock) = pallas_codec::minicbor::decode(cbor)?;
        self.crawl_conway_with_cbor(&block, cbor)
    }
}
