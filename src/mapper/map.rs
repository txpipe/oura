use std::collections::HashMap;

use minicbor::bytes::ByteVec;
use pallas::crypto::hash::Hash;
use pallas::ledger::alonzo::{
    self as alonzo, AuxiliaryData, Block, Certificate, InstantaneousRewardSource,
    InstantaneousRewardTarget, Metadatum, Relay, TransactionInput, TransactionOutput, Value,
};
use pallas::ledger::alonzo::{NetworkId, TransactionBody, TransactionBodyComponent};

use serde_json::{json, Value as JsonValue};

use crate::model::{
    BlockRecord, EventData, MetadataRecord, MetadatumRendition, MintRecord, OutputAssetRecord,
    StakeCredential, TransactionRecord, TxInputRecord, TxOutputRecord,
};

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

fn metadatum_to_string_key(datum: &Metadatum) -> Result<String, Error> {
    match datum {
        Metadatum::Int(x) => Ok(x.to_string()),
        Metadatum::Bytes(x) => Ok(hex::encode(x.as_slice())),
        Metadatum::Text(x) => Ok(x.to_owned()),
        _ => Err("unexpected metadatum type for label".into()),
    }
}

fn get_tx_output_coin_value(amount: &Value) -> u64 {
    match amount {
        Value::Coin(x) => *x,
        Value::Multiasset(x, _) => *x,
    }
}

impl EventWriter {
    pub fn to_metadatum_json_map_entry(
        &self,
        pair: (&Metadatum, &Metadatum),
    ) -> Result<(String, JsonValue), Error> {
        let key = metadatum_to_string_key(pair.0)?;
        let value = self.to_metadatum_json(pair.1)?;
        Ok((key, value))
    }

    pub fn to_metadatum_json(&self, source: &Metadatum) -> Result<JsonValue, Error> {
        match source {
            Metadatum::Int(x) => Ok(json!(x)),
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
        label: &Metadatum,
        value: &Metadatum,
    ) -> Result<MetadataRecord, Error> {
        let data = MetadataRecord {
            label: metadatum_to_string_key(label)?,
            content: match value {
                Metadatum::Int(x) => MetadatumRendition::IntScalar(*x),
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

    pub fn to_transaction_output_record(
        &self,
        output: &TransactionOutput,
    ) -> Result<TxOutputRecord, Error> {
        Ok(TxOutputRecord {
            address: self
                .utils
                .bech32
                .encode_address(output.address.as_slice())?,
            amount: get_tx_output_coin_value(&output.amount),
            assets: self.collect_asset_records(&output.amount).into(),
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

    pub fn to_native_script_event(&self) -> EventData {
        EventData::NativeScript {}
    }

    pub fn to_plutus_script_event(&self, script: &ByteVec) -> EventData {
        EventData::PlutusScript {
            data: script.to_hex(),
        }
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
                pledge: *pledge,
                cost: *cost,
                margin: (margin.numerator as f64 / margin.denominator as f64),
                reward_account: reward_account.to_hex(),
                pool_owners: pool_owners.iter().map(|p| p.to_hex()).collect(),
                relays: relays.iter().map(relay_to_string).collect(),
                pool_metadata: pool_metadata.as_ref().map(|m| m.url.clone()),
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
                        InstantaneousRewardTarget::OtherAccountingPot(x) => Some(x),
                        _ => None,
                    },
                }
            }

            // TODO: not likely, leaving for later
            Certificate::GenesisKeyDelegation(..) => EventData::GenesisKeyDelegation,
        }
    }

    pub fn to_collateral_event(&self, collateral: &TransactionInput) -> EventData {
        EventData::Collateral {
            tx_id: collateral.transaction_id.to_hex(),
            index: collateral.index,
        }
    }

    pub fn to_transaction_record(
        &self,
        body: &TransactionBody,
        tx_hash: &str,
        aux_data: Option<&AuxiliaryData>,
    ) -> Result<TransactionRecord, Error> {
        let mut record = TransactionRecord::default();

        record.hash.push_str(tx_hash);

        for component in body.iter() {
            match component {
                TransactionBodyComponent::Fee(x) => {
                    record.fee = *x;
                }
                TransactionBodyComponent::Ttl(x) => {
                    record.ttl = Some(*x);
                }
                TransactionBodyComponent::ValidityIntervalStart(x) => {
                    record.validity_interval_start = Some(*x);
                }
                TransactionBodyComponent::NetworkId(x) => {
                    record.network_id = match x {
                        NetworkId::One => Some(1),
                        NetworkId::Two => Some(2),
                    };
                }
                TransactionBodyComponent::Outputs(x) => {
                    let sub_records = self.collect_output_records(x)?;
                    record.output_count = sub_records.len();
                    record.total_output = sub_records.iter().map(|o| o.amount).sum();

                    if self.config.include_transaction_details {
                        record.outputs = sub_records.into();
                    }
                }
                TransactionBodyComponent::Inputs(x) => {
                    let sub_records = self.collect_input_records(x);
                    record.input_count = sub_records.len();

                    if self.config.include_transaction_details {
                        record.inputs = sub_records.into();
                    }
                }
                TransactionBodyComponent::Mint(x) => {
                    let sub_records = self.collect_mint_records(x);
                    record.mint_count = sub_records.len();

                    if self.config.include_transaction_details {
                        record.mint = sub_records.into();
                    }
                }
                // TODO
                // TransactionBodyComponent::ScriptDataHash(_) => todo!(),
                // TransactionBodyComponent::RequiredSigners(_) => todo!(),
                // TransactionBodyComponent::AuxiliaryDataHash(_) => todo!(),
                _ => (),
            };
        }

        // TODO: add witness set data to transaction
        /*
        if let Some(witness) = self.transaction_witness_sets.get(idx) {
            let plutus_count = match &witness.plutus_script {
                Some(scripts) => scripts.len(),
                None => 0,
            };

            let native_count = match &witness.native_script {
                Some(scripts) => scripts.len(),
                None => 0,
            };

            let redeemer_count = match &witness.redeemer {
                Some(redeemer) => redeemer.len(),
                None => 0,
            };
        }
        */

        if self.config.include_transaction_details {
            record.metadata = match aux_data {
                Some(aux_data) => self.collect_metadata_records(aux_data)?.into(),
                None => None,
            };
        }

        Ok(record)
    }

    pub fn to_block_record(&self, source: &Block, hash: &Hash<32>) -> Result<BlockRecord, Error> {
        Ok(BlockRecord {
            body_size: source.header.header_body.block_body_size as usize,
            issuer_vkey: source.header.header_body.issuer_vkey.to_hex(),
            tx_count: source.transaction_bodies.len(),
            hash: hex::encode(hash),
            number: source.header.header_body.block_number,
            slot: source.header.header_body.slot,
            previous_hash: hex::encode(source.header.header_body.prev_hash),
        })
    }
}
