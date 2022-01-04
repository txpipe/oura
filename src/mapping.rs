use std::collections::HashMap;

use pallas::ledger::alonzo::{
    self as alonzo, crypto::hash_transaction, AuxiliaryData, Block, Certificate,
    InstantaneousRewardSource, InstantaneousRewardTarget, Metadata, Metadatum, Relay,
    TransactionInput, TransactionOutput, Value,
};
use pallas::ledger::alonzo::{
    AlonzoAuxiliaryData, Mint, NetworkId, TransactionBody, TransactionBodyComponent,
};

use bech32::{self, ToBase32};
use serde_derive::Deserialize;

use serde_json::{Value as JsonValue, json};

use crate::framework::{
    EventContext, EventData, EventSource, EventWriter, MetadataRecord, MintRecord,
    OutputAssetRecord, StakeCredential, TransactionRecord, TxInputRecord, TxOutputRecord,
};

use crate::framework::Error;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct MapperConfig {
    #[serde(default)]
    pub include_transaction_details: bool,
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Vec<u8> {
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

pub trait ToBech32 {
    fn try_to_bech32(&self, hrp: &str) -> Result<String, Error>;
}

impl ToBech32 for Vec<u8> {
    fn try_to_bech32(&self, hrp: &str) -> Result<String, Error> {
        let enc = bech32::encode(hrp, self.to_base32(), bech32::Variant::Bech32)?;
        Ok(enc)
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

impl EventSource for Certificate {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        let event = match self {
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
        };

        writer.append(event)?;

        Ok(())
    }
}

fn metadatum_to_string_key(datum: &Metadatum) -> Result<String, Error> {
    match datum {
        Metadatum::Int(x) => Ok(x.to_string()),
        Metadatum::Bytes(x) => Ok(hex::encode(x.as_slice())),
        Metadatum::Text(x) => Ok(x.to_owned()),
        _ => Err("can't turn complex metadatums into string keys".into()),
    }
}

fn metadatum_map_entry_to_json_map_entry(
    pair: (&Metadatum, &Metadatum),
) -> Result<(String, JsonValue), Error> {
    let key = metadatum_to_string_key(pair.0)?;
    let value = metadatum_to_json(pair.1)?;
    Ok((key, value))
}

fn metadatum_to_json(source: &Metadatum) -> Result<JsonValue, Error> {
    match source {
        Metadatum::Int(x) => Ok(json!(x)),
        Metadatum::Bytes(x) => Ok(json!(hex::encode(x.as_slice()))),
        Metadatum::Text(x) => Ok(json!(x)),
        Metadatum::Array(x) => {
            let items: Result<Vec<_>, _> = x.iter().map(|i| metadatum_to_json(i)).collect();

            Ok(json!(items?))
        }
        Metadatum::Map(x) => {
            let map: Result<HashMap<_, _>, _> = x
                .iter()
                .map(|(key, value)| metadatum_map_entry_to_json_map_entry((key, value)))
                .collect();

            Ok(json!(map?))
        }
    }
}

trait MetadataProvider {
    fn try_get_metadata(&self) -> Result<Vec<MetadataRecord>, Error>;
}

impl TryFrom<(&Metadatum, &Metadatum)> for MetadataRecord {
    type Error = Error;

    fn try_from(value: (&Metadatum, &Metadatum)) -> Result<Self, Self::Error> {
        Ok(MetadataRecord {
            label: metadatum_to_string_key(value.0)?,
            content: metadatum_to_json(value.1)?,
        })
    }
}

impl MetadataProvider for &Metadata {
    fn try_get_metadata(&self) -> Result<Vec<MetadataRecord>, Error> {
        let out: Result<Vec<_>, Error> = self.iter()
            .map(|(key, value)| MetadataRecord::try_from((key, value)))
            .collect();

        Ok(out?)
    }
}

impl EventSource for Metadata {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        for record in self.try_get_metadata()? {
            writer.append(EventData::Metadata(record))?;
        }

        Ok(())
    }
}

impl EventSource for AuxiliaryData {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        match self {
            AuxiliaryData::Alonzo(data) => {
                if let Some(metadata) = &data.metadata {
                    metadata.write_events(writer)?;
                }

                for _native in data.native_scripts.iter() {
                    writer.append(EventData::NativeScript)?;
                }

                for plutus in data.plutus_scripts.iter() {
                    writer.append(EventData::PlutusScript {
                        data: plutus.to_hex(),
                    })?;
                }
            }
            AuxiliaryData::Shelley(data) => {
                data.write_events(writer)?;
            }
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                ..
            } => {
                transaction_metadata.write_events(writer)?;

                // TODO: process auxiliary scripts
            }
        }

        Ok(())
    }
}

impl MetadataProvider for AuxiliaryData {
    fn try_get_metadata(&self) -> Result<Vec<MetadataRecord>, Error> {
        match self {
            AuxiliaryData::Alonzo(AlonzoAuxiliaryData {
                metadata: Some(metadata),
                ..
            }) => metadata.try_get_metadata(),
            AuxiliaryData::Shelley(metadata) => metadata.try_get_metadata(),
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                ..
            } => transaction_metadata.try_get_metadata(),
            _ => Ok(vec![]),
        }
    }
}

fn get_tx_output_coin_value(amount: &Value) -> u64 {
    match amount {
        Value::Coin(x) => *x,
        Value::Multiasset(x, _) => *x,
    }
}

trait AssetProvider {
    fn get_assets(&self) -> Vec<OutputAssetRecord>;
}

impl AssetProvider for TransactionOutput {
    fn get_assets(&self) -> Vec<OutputAssetRecord> {
        match &self.amount {
            Value::Multiasset(_, assets) => {
                let mut out = Vec::with_capacity(assets.len());

                for (policy, assets) in assets.iter() {
                    for (asset, amount) in assets.iter() {
                        out.push(OutputAssetRecord {
                            policy: policy.to_hex(),
                            asset: asset.to_hex(),
                            amount: *amount,
                        });
                    }
                }

                out
            }
            _ => vec![],
        }
    }
}

trait TxOutputProvider {
    fn collect_outputs(&self) -> Result<Vec<TxOutputRecord>, Error>;
}

impl TxOutputProvider for [TransactionOutput] {
    fn collect_outputs(&self) -> Result<Vec<TxOutputRecord>, Error> {
        self.iter()
            .map(|o| {
                Ok(TxOutputRecord {
                    address: o.address.try_to_bech32("addr")?,
                    amount: get_tx_output_coin_value(&o.amount),
                    assets: o.get_assets().into(),
                })
            })
            .collect()
    }
}

trait TxInputProvider {
    fn collect_inputs(&self) -> Result<Vec<TxInputRecord>, Error>;
}

impl TxInputProvider for [TransactionInput] {
    fn collect_inputs(&self) -> Result<Vec<TxInputRecord>, Error> {
        self.iter()
            .map(|i| {
                Ok(TxInputRecord {
                    tx_id: i.transaction_id.to_hex(),
                    index: i.index,
                })
            })
            .collect()
    }
}

trait MintProvider {
    fn collect_mint(&self) -> Result<Vec<MintRecord>, Error>;
}

impl MintProvider for Mint {
    fn collect_mint(&self) -> Result<Vec<MintRecord>, Error> {
        let out: Vec<_> = self
            .iter()
            .flat_map(|(policy, value)| {
                value.iter().map(|(asset, quantity)| MintRecord {
                    policy: policy.to_hex(),
                    asset: asset.to_hex(),
                    quantity: *quantity,
                })
            })
            .collect();

        Ok(out)
    }
}

impl EventSource for TransactionBodyComponent {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        match self {
            TransactionBodyComponent::Inputs(x) => {
                let inputs = x.as_slice().collect_inputs()?;

                for (idx, input) in inputs.into_iter().enumerate() {
                    let writer = writer.child_writer(EventContext {
                        input_idx: Some(idx),
                        ..EventContext::default()
                    });

                    writer.append(EventData::TxInput(input))?;
                }
            }
            TransactionBodyComponent::Outputs(outputs) => {
                let outputs = outputs.as_slice().collect_outputs()?;

                for (idx, output) in outputs.into_iter().enumerate() {
                    let writer = writer.child_writer(EventContext {
                        output_idx: Some(idx),
                        ..EventContext::default()
                    });

                    writer.append(EventData::TxOutput(output.clone()))?;

                    if let Some(assets) = output.assets {
                        if !assets.is_empty() {
                            let writer = &writer.child_writer(EventContext {
                                output_address: output.address.clone().into(),
                                ..EventContext::default()
                            });

                            for asset in assets {
                                writer.append(EventData::OutputAsset(asset))?;
                            }
                        }
                    }
                }
            }
            TransactionBodyComponent::Certificates(certs) => {
                for (idx, cert) in certs.iter().enumerate() {
                    let writer = writer.child_writer(EventContext {
                        certificate_idx: Some(idx),
                        ..EventContext::default()
                    });

                    cert.write_events(&writer)?;
                }
            }
            TransactionBodyComponent::Mint(mint) => {
                let records = mint.collect_mint()?;

                for record in records {
                    writer.append(EventData::Mint(record))?;
                }
            }
            TransactionBodyComponent::Collateral(collaterals) => {
                for collateral in collaterals.iter() {
                    writer.append(EventData::Collateral {
                        tx_id: collateral.transaction_id.to_hex(),
                        index: collateral.index,
                    })?;
                }
            }
            _ => (),
        };

        Ok(())
    }
}

impl EventSource for (&TransactionBody, Option<&AuxiliaryData>) {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        let (body, aux_data) = self;

        let mut record = TransactionRecord::default();

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
                    let sub_records = x.as_slice().collect_outputs()?;
                    record.output_count = sub_records.len();
                    record.total_output = sub_records.iter().map(|o| o.amount).sum();

                    if writer.mapping_config.include_transaction_details {
                        record.outputs = sub_records.into();
                    }
                }
                TransactionBodyComponent::Inputs(x) => {
                    let sub_records = x.as_slice().collect_inputs()?;
                    record.input_count = sub_records.len();

                    if writer.mapping_config.include_transaction_details {
                        record.inputs = sub_records.into();
                    }
                }
                TransactionBodyComponent::Mint(x) => {
                    let sub_records = x.collect_mint()?;
                    record.mint_count = sub_records.len();

                    if writer.mapping_config.include_transaction_details {
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

        if writer.mapping_config.include_transaction_details {
            record.metadata = match aux_data {
                Some(aux_data) => Some(aux_data.try_get_metadata()?),
                None => None,
            };
        }

        writer.append(EventData::Transaction(record))?;

        // write aux data custom events
        if let Some(aux_data) = aux_data {
            aux_data.write_events(writer)?;
        }

        // write body components sub-events
        for component in body.iter() {
            component.write_events(writer)?;
        }

        Ok(())
    }
}

impl EventSource for Block {
    fn write_events(&self, writer: &EventWriter) -> Result<(), Error> {
        let writer = writer.child_writer(EventContext {
            block_number: Some(self.header.header_body.block_number),
            slot: Some(self.header.header_body.slot),
            timestamp: writer.compute_timestamp(self.header.header_body.slot),
            ..EventContext::default()
        });

        writer.append(EventData::Block {
            body_size: self.header.header_body.block_body_size as usize,
            issuer_vkey: self.header.header_body.issuer_vkey.to_hex(),
            tx_count: self.transaction_bodies.len(),
        })?;

        for (idx, tx) in self.transaction_bodies.iter().enumerate() {
            let tx_hash = match hash_transaction(tx) {
                Ok(h) => Some(hex::encode(h)),
                Err(err) => {
                    log::warn!("error hashing transaction: {:?}", err);
                    None
                }
            };

            let writer = writer.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_hash: tx_hash.clone(),
                ..EventContext::default()
            });

            let aux_data = self
                .auxiliary_data_set
                .iter()
                .find(|(k, _)| *k == (idx as u32))
                .map(|(_, v)| v);

            (tx, aux_data).write_events(&writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ToBech32;

    #[test]
    fn beach32_encodes_ok() {
        let bytes = hex::decode("01ec6ad5daee9febbe300c6160a36d4daf0c5266ae2fe8245cbb581390629814d8165fd547b6f3f6f55842a5f042bcb113e8e86627bc071f37").unwrap();
        let bech32 = bytes.try_to_bech32("addr").unwrap();

        assert_eq!(bech32, "addr1q8kx44w6a607h03sp3skpgmdfkhsc5nx4ch7sfzuhdvp8yrznq2ds9jl64rmdulk74vy9f0sg27tzylgapnz00q8rumsuhj834");
    }
}
