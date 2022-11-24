use std::collections::HashMap;

use pallas::{
    codec::{minicbor::bytes::ByteVec, utils::KeepRaw},
    crypto::hash::Hash,
    ledger::primitives::babbage::DatumOption,
};

use pallas::ledger::primitives::{
    alonzo::{
        self as alonzo, AuxiliaryData, Certificate, InstantaneousRewardSource,
        InstantaneousRewardTarget, Metadatum, MetadatumLabel, MintedBlock, NetworkId, Relay,
        TransactionBody, TransactionInput, TransactionWitnessSet, Value,
    },
    babbage, ToCanonicalJson, ToHash,
};

use pallas::network::miniprotocols::Point;
use serde_json::{json, Value as JsonValue};

use crate::model::{
    BlockRecord, Era, EventData, MetadataRecord, MetadatumRendition, MintRecord,
    NativeWitnessRecord, OutputAssetRecord, PlutusDatumRecord, PlutusRedeemerRecord,
    PlutusWitnessRecord, StakeCredential, TransactionRecord, TxInputRecord, TxOutputRecord,
    VKeyWitnessRecord,
};

use crate::utils::time::TimeProvider;
use crate::Error;

use super::EventWriter;

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl ToHex for &[u8] {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl<const BYTES: usize> ToHex for Hash<BYTES> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl From<&alonzo::StakeCredential> for StakeCredential {
    fn from(other: &alonzo::StakeCredential) -> Self {
        match other {
            alonzo::StakeCredential::AddrKeyhash(x) => StakeCredential::AddrKeyhash(x.to_hex()),
            alonzo::StakeCredential::Scripthash(x) => StakeCredential::Scripthash(x.to_hex()),
        }
    }
}

fn ip_string_from_bytes(bytes: &[u8]) -> String {
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

fn relay_to_string(relay: &Relay) -> String {
    match relay {
        Relay::SingleHostAddr(port, ipv4, ipv6) => {
            let ip = match (ipv6, ipv4) {
                (None, None) => "".to_string(),
                (_, Some(x)) => ip_string_from_bytes(x.as_ref()),
                (Some(x), _) => ip_string_from_bytes(x.as_ref()),
            };

            match port {
                Some(port) => format!("{}:{}", ip, port),
                None => ip,
            }
        }
        Relay::SingleHostName(port, host) => match port {
            Some(port) => format!("{}:{}", host, port),
            None => host.clone(),
        },
        Relay::MultiHostName(host) => host.clone(),
    }
}

fn metadatum_to_string_key(datum: &Metadatum) -> String {
    match datum {
        Metadatum::Int(x) => x.to_string(),
        Metadatum::Bytes(x) => hex::encode(x.as_slice()),
        Metadatum::Text(x) => x.to_owned(),
        x => {
            log::warn!("unexpected metadatum type for label: {:?}", x);
            Default::default()
        }
    }
}

fn get_tx_output_coin_value(amount: &Value) -> u64 {
    match amount {
        Value::Coin(x) => x.into(),
        Value::Multiasset(x, _) => x.into(),
    }
}

impl EventWriter {
    pub fn to_metadatum_json_map_entry(
        &self,
        pair: (&Metadatum, &Metadatum),
    ) -> Result<(String, JsonValue), Error> {
        let key = metadatum_to_string_key(pair.0);
        let value = self.to_metadatum_json(pair.1)?;
        Ok((key, value))
    }

    pub fn to_metadatum_json(&self, source: &Metadatum) -> Result<JsonValue, Error> {
        match source {
            Metadatum::Int(x) => Ok(json!(i128::from(*x))),
            Metadatum::Bytes(x) => Ok(json!(hex::encode(x.as_slice()))),
            Metadatum::Text(x) => Ok(json!(x)),
            Metadatum::Array(x) => {
                let items: Result<Vec<_>, _> =
                    x.iter().map(|x| self.to_metadatum_json(x)).collect();

                Ok(json!(items?))
            }
            Metadatum::Map(x) => {
                let map: Result<HashMap<_, _>, _> = x
                    .iter()
                    .map(|(key, value)| self.to_metadatum_json_map_entry((key, value)))
                    .collect();

                Ok(json!(map?))
            }
        }
    }

    pub fn to_metadata_record(
        &self,
        label: &MetadatumLabel,
        value: &Metadatum,
    ) -> Result<MetadataRecord, Error> {
        let data = MetadataRecord {
            label: u64::from(label).to_string(),
            content: match value {
                Metadatum::Int(x) => MetadatumRendition::IntScalar(i128::from(*x)),
                Metadatum::Bytes(x) => MetadatumRendition::BytesHex(hex::encode(x.as_slice())),
                Metadatum::Text(x) => MetadatumRendition::TextScalar(x.clone()),
                Metadatum::Array(_) => {
                    MetadatumRendition::ArrayJson(self.to_metadatum_json(value)?)
                }
                Metadatum::Map(_) => MetadatumRendition::MapJson(self.to_metadatum_json(value)?),
            },
        };

        Ok(data)
    }

    pub fn to_transaction_input_record(&self, input: &TransactionInput) -> TxInputRecord {
        TxInputRecord {
            tx_id: input.transaction_id.to_hex(),
            index: input.index,
        }
    }

    pub fn to_legacy_output_record(
        &self,
        output: &alonzo::TransactionOutput,
    ) -> Result<TxOutputRecord, Error> {
        Ok(TxOutputRecord {
            address: self
                .utils
                .bech32
                .encode_address(output.address.as_slice())?,
            amount: get_tx_output_coin_value(&output.amount),
            assets: self.collect_asset_records(&output.amount).into(),
            datum_hash: output.datum_hash.map(|hash| hash.to_string()),
        })
    }

    pub fn to_post_alonzo_output_record(
        &self,
        output: &babbage::PostAlonzoTransactionOutput,
    ) -> Result<TxOutputRecord, Error> {
        Ok(TxOutputRecord {
            address: self
                .utils
                .bech32
                .encode_address(output.address.as_slice())?,
            amount: get_tx_output_coin_value(&output.value),
            assets: self.collect_asset_records(&output.value).into(),
            datum_hash: match output.datum_option {
                Some(DatumOption::Hash(x)) => Some(x.to_string()),
                _ => None,
            },
        })
    }

    pub fn to_transaction_output_asset_record(
        &self,
        policy: &ByteVec,
        asset: &ByteVec,
        amount: u64,
    ) -> OutputAssetRecord {
        OutputAssetRecord {
            policy: policy.to_hex(),
            asset: asset.to_hex(),
            asset_ascii: String::from_utf8(asset.to_vec()).ok(),
            amount,
        }
    }

    pub fn to_mint_record(&self, policy: &ByteVec, asset: &ByteVec, quantity: i64) -> MintRecord {
        MintRecord {
            policy: policy.to_hex(),
            asset: asset.to_hex(),
            quantity,
        }
    }

    pub fn to_aux_native_script_event(&self, script: &alonzo::NativeScript) -> EventData {
        EventData::NativeScript {
            policy_id: script.to_hash().to_hex(),
            script: script.to_json(),
        }
    }

    pub fn to_aux_plutus_script_event(&self, script: &alonzo::PlutusScript) -> EventData {
        EventData::PlutusScript {
            hash: script.to_hash().to_hex(),
            data: script.0.to_hex(),
        }
    }

    pub fn to_plutus_redeemer_record(
        &self,
        redeemer: &alonzo::Redeemer,
    ) -> Result<PlutusRedeemerRecord, crate::Error> {
        Ok(PlutusRedeemerRecord {
            purpose: match redeemer.tag {
                alonzo::RedeemerTag::Spend => "spend".to_string(),
                alonzo::RedeemerTag::Mint => "mint".to_string(),
                alonzo::RedeemerTag::Cert => "cert".to_string(),
                alonzo::RedeemerTag::Reward => "reward".to_string(),
            },
            ex_units_mem: redeemer.ex_units.mem,
            ex_units_steps: redeemer.ex_units.steps,
            input_idx: redeemer.index,
            plutus_data: redeemer.data.to_json(),
        })
    }

    pub fn to_plutus_datum_record(
        &self,
        datum: &alonzo::PlutusData,
    ) -> Result<PlutusDatumRecord, crate::Error> {
        Ok(PlutusDatumRecord {
            datum_hash: datum.to_hash().to_hex(),
            plutus_data: datum.to_json(),
        })
    }

    pub fn to_plutus_v1_witness_record(
        &self,
        script: &alonzo::PlutusScript,
    ) -> Result<PlutusWitnessRecord, crate::Error> {
        Ok(PlutusWitnessRecord {
            script_hash: script.to_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        })
    }

    pub fn to_plutus_v2_witness_record(
        &self,
        script: &babbage::PlutusV2Script,
    ) -> Result<PlutusWitnessRecord, crate::Error> {
        Ok(PlutusWitnessRecord {
            script_hash: script.to_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        })
    }

    pub fn to_native_witness_record(
        &self,
        script: &alonzo::NativeScript,
    ) -> Result<NativeWitnessRecord, crate::Error> {
        Ok(NativeWitnessRecord {
            policy_id: script.to_hash().to_hex(),
            script_json: script.to_json(),
        })
    }

    pub fn to_vkey_witness_record(
        &self,
        witness: &alonzo::VKeyWitness,
    ) -> Result<VKeyWitnessRecord, crate::Error> {
        Ok(VKeyWitnessRecord {
            vkey_hex: witness.vkey.to_hex(),
            signature_hex: witness.signature.to_hex(),
        })
    }

    pub fn to_certificate_event(&self, certificate: &Certificate) -> EventData {
        match certificate {
            Certificate::StakeRegistration(credential) => EventData::StakeRegistration {
                credential: credential.into(),
            },
            Certificate::StakeDeregistration(credential) => EventData::StakeDeregistration {
                credential: credential.into(),
            },
            Certificate::StakeDelegation(credential, pool) => EventData::StakeDelegation {
                credential: credential.into(),
                pool_hash: pool.to_hex(),
            },
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
                pledge: pledge.into(),
                cost: cost.into(),
                margin: (margin.numerator as f64 / margin.denominator as f64),
                reward_account: reward_account.to_hex(),
                pool_owners: pool_owners.iter().map(|p| p.to_hex()).collect(),
                relays: relays.iter().map(relay_to_string).collect(),
                pool_metadata: pool_metadata.as_ref().map(|m| m.url.clone()),
                pool_metadata_hash: pool_metadata.as_ref().map(|m| m.hash.clone().to_hex()),
            },
            Certificate::PoolRetirement(pool, epoch) => EventData::PoolRetirement {
                pool: pool.to_hex(),
                epoch: *epoch,
            },
            Certificate::MoveInstantaneousRewardsCert(move_) => {
                EventData::MoveInstantaneousRewardsCert {
                    from_reserves: matches!(move_.source, InstantaneousRewardSource::Reserves),
                    from_treasury: matches!(move_.source, InstantaneousRewardSource::Treasury),
                    to_stake_credentials: match &move_.target {
                        InstantaneousRewardTarget::StakeCredentials(creds) => {
                            let x = creds.iter().map(|(k, v)| (k.into(), *v)).collect();
                            Some(x)
                        }
                        _ => None,
                    },
                    to_other_pot: match move_.target {
                        InstantaneousRewardTarget::OtherAccountingPot(x) => Some(x.into()),
                        _ => None,
                    },
                }
            }
            // TODO: not likely, leaving for later
            Certificate::GenesisKeyDelegation(..) => EventData::GenesisKeyDelegation {},
        }
    }

    pub fn to_collateral_event(&self, collateral: &TransactionInput) -> EventData {
        EventData::Collateral {
            tx_id: collateral.transaction_id.to_hex(),
            index: collateral.index,
        }
    }

    pub fn to_tx_size(
        &self,
        body: &KeepRaw<TransactionBody>,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<TransactionWitnessSet>>,
    ) -> usize {
        body.raw_cbor().len()
            + aux_data.map(|ax| ax.raw_cbor().len()).unwrap_or(2)
            + witness_set.map(|ws| ws.raw_cbor().len()).unwrap_or(1)
    }

    pub fn to_transaction_record(
        &self,
        body: &KeepRaw<TransactionBody>,
        tx_hash: &str,
        aux_data: Option<&KeepRaw<AuxiliaryData>>,
        witness_set: Option<&KeepRaw<TransactionWitnessSet>>,
    ) -> Result<TransactionRecord, Error> {
        let mut record = TransactionRecord {
            hash: tx_hash.to_owned(),
            size: self.to_tx_size(body, aux_data, witness_set) as u32,
            fee: body.fee,
            ttl: body.ttl,
            validity_interval_start: body.validity_interval_start,
            network_id: body.network_id.as_ref().map(|x| match x {
                NetworkId::One => 1,
                NetworkId::Two => 2,
            }),
            ..TransactionRecord::default()
        };

        let outputs = self.collect_legacy_output_records(&body.outputs)?;
        record.output_count = outputs.len();
        record.total_output = outputs.iter().map(|o| o.amount).sum();

        let inputs = self.collect_input_records(&body.inputs);
        record.input_count = inputs.len();

        if let Some(mint) = &body.mint {
            let mints = self.collect_mint_records(mint);
            record.mint_count = mints.len();

            if self.config.include_transaction_details {
                record.mint = mints.into();
            }
        }

        // TODO
        // TransactionBodyComponent::ScriptDataHash(_)
        // TransactionBodyComponent::RequiredSigners(_)
        // TransactionBodyComponent::AuxiliaryDataHash(_)

        if self.config.include_transaction_details {
            record.outputs = outputs.into();
            record.inputs = inputs.into();

            record.metadata = match aux_data {
                Some(aux_data) => self.collect_metadata_records(aux_data)?.into(),
                None => None,
            };

            if let Some(witnesses) = witness_set {
                record.vkey_witnesses = self
                    .collect_vkey_witness_records(&witnesses.vkeywitness)?
                    .into();

                record.native_witnesses = self
                    .collect_native_witness_records(&witnesses.native_script)?
                    .into();

                record.plutus_witnesses = self
                    .collect_plutus_v1_witness_records(&witnesses.plutus_script)?
                    .into();

                record.plutus_redeemers = self
                    .collect_plutus_redeemer_records(&witnesses.redeemer)?
                    .into();

                record.plutus_data = self
                    .collect_plutus_datum_records(&witnesses.plutus_data)?
                    .into();
            }

            if let Some(withdrawals) = &body.withdrawals {
                record.withdrawals = self.collect_withdrawal_records(withdrawals).into();
            }
        }

        Ok(record)
    }

    pub fn to_block_record(
        &self,
        source: &MintedBlock,
        hash: &Hash<32>,
        cbor: &[u8],
        era: Era,
    ) -> Result<BlockRecord, Error> {
        let relative_epoch = self
            .utils
            .time
            .as_ref()
            .map(|time| time.absolute_slot_to_relative(source.header.header_body.slot));

        let mut record = BlockRecord {
            era,
            body_size: source.header.header_body.block_body_size as usize,
            issuer_vkey: source.header.header_body.issuer_vkey.to_hex(),
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

        if self.config.include_block_details {
            record.transactions = Some(self.collect_shelley_tx_records(source)?);
        }

        Ok(record)
    }

    pub(crate) fn append_rollback_event(&self, point: &Point) -> Result<(), Error> {
        let data = match point {
            Point::Origin => EventData::RollBack {
                block_slot: 0,
                block_hash: "".to_string(),
            },
            Point::Specific(slot, hash) => EventData::RollBack {
                block_slot: *slot,
                block_hash: hex::encode(&hash),
            },
        };

        self.append(data)
    }
}
