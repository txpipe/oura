use std::{collections::BTreeMap, ops::Deref};

use merge::Merge;
use pallas::ledger::alonzo::{
    AuxiliaryData, Block, Certificate, Metadatum, TransactionOutput, Value,
};

use crate::ports::{Event, EventContext, EventData};

pub type Storage = Vec<Event>;

trait ToHex {
    fn to_hex(&self) -> String;
}

impl<T> ToHex for T
where
    T: Deref<Target = Vec<u8>>,
{
    fn to_hex(&self) -> String {
        hex::encode(self.deref())
    }
}

pub struct EventWriter<'a> {
    context: EventContext,
    storage: &'a mut Storage,
}

impl<'a> EventWriter<'a> {
    pub fn new(storage: &mut Storage) -> EventWriter<'_> {
        EventWriter {
            context: EventContext::default(),
            storage,
        }
    }

    fn append(&mut self, data: EventData) -> &mut Self {
        self.storage.push(Event {
            context: self.context.clone(),
            data,
        });

        self
    }

    fn child_writer(&mut self, mut extra_context: EventContext) -> EventWriter<'_> {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            storage: self.storage,
        }
    }
}

pub trait EventSource {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter);
}

impl EventSource for Certificate {
    fn write_events(&self, writer: &mut EventWriter) {
        let event = match self {
            Certificate::StakeRegistration(..) => EventData::StakeRegistration,
            Certificate::StakeDeregistration(..) => EventData::StakeDeregistration,
            Certificate::StakeDelegation(..) => EventData::StakeDelegation,
            Certificate::PoolRegistration { .. } => EventData::PoolRegistration,
            Certificate::PoolRetirement(..) => EventData::PoolRetirement,
            Certificate::GenesisKeyDelegation(..) => EventData::GenesisKeyDelegation,
            Certificate::MoveInstantaneousRewardsCert(..) => {
                EventData::MoveInstantaneousRewardsCert
            }
        };

        writer.append(event);
    }
}

fn metadatum_to_string(datum: &Metadatum) -> String {
    match datum {
        Metadatum::Int(x) => x.to_string(),
        Metadatum::Bytes(x) => hex::encode::<&Vec<u8>>(x.as_ref()),
        Metadatum::Text(x) => x.to_owned(),
        Metadatum::Array(x) => x
            .iter()
            .map(|i| format!("{}, ", metadatum_to_string(i)))
            .collect(),
        Metadatum::Map(x) => x
            .iter()
            .map(|(key, val)| {
                format!(
                    "{}: {}, ",
                    metadatum_to_string(key),
                    metadatum_to_string(val)
                )
            })
            .collect(),
    }
}

impl EventSource for BTreeMap<Metadatum, Metadatum> {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        for (level1_key, level1_data) in self {
            match level1_data {
                Metadatum::Map(level1_map) => {
                    for (level2_key, level2_data) in level1_map {
                        writer.append(EventData::Metadata {
                            key: metadatum_to_string(level1_key),
                            subkey: Some(metadatum_to_string(level2_key)),
                            value: Some(metadatum_to_string(level2_data)),
                        });
                    }
                }
                _ => {
                    writer.append(EventData::Metadata {
                        key: metadatum_to_string(level1_key),
                        subkey: None,
                        value: None,
                    });
                }
            }
        }
    }
}

impl EventSource for AuxiliaryData {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        match self {
            AuxiliaryData::Alonzo(data) => {
                if let Some(metadata) = &data.metadata {
                    metadata.write_events(writer);
                }

                for _native in data.native_scripts.iter() {
                    writer.append(EventData::NewNativeScript);
                }

                for plutus in data.plutus_scripts.iter() {
                    writer.append(EventData::NewPlutusScript {
                        data: plutus.to_hex(),
                    });
                }
            }
            AuxiliaryData::Shelley(data) => {
                data.write_events(writer);
            }
            _ => log::warn!("ShelleyMa auxiliary data, not sure what to do"),
        }
    }
}

impl EventSource for TransactionOutput {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        writer.append(EventData::TxOutput {
            address: self.address.to_hex(),
            amount: match self.amount {
                Value::Coin(x) => x,
                Value::Multiasset(x, _) => x,
            },
        });

        if let Value::Multiasset(_, assets) = &self.amount {
            for (policy, assets) in assets.iter() {
                for (asset, amount) in assets.iter() {
                    writer.append(EventData::OutputAsset {
                        policy: policy.to_hex(),
                        asset: asset.to_hex(),
                        amount: *amount,
                    });
                }
            }
        }
    }
}

impl EventSource for Block {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        let mut writer = writer.child_writer(EventContext {
            block_number: Some(self.header.header_body.block_number),
            slot: Some(self.header.header_body.slot),
            ..EventContext::default()
        });

        writer.append(EventData::Block {
            body_size: self.header.header_body.block_body_size as usize,
            issuer_vkey: self.header.header_body.issuer_vkey.to_hex(),
        });

        for (idx, tx) in self.transaction_bodies.iter().enumerate() {
            let mut writer = writer.child_writer(EventContext {
                tx_idx: Some(idx),
                tx_id: Some("some-hash".to_string()),
                ..EventContext::default()
            });

            writer.append(EventData::Transaction {
                fee: tx.fee,
                ttl: tx.ttl,
                validity_interval_start: tx.validity_interval_start,
            });

            if let Some(mint) = &tx.mint {
                for (policy, value) in mint.iter() {
                    for (asset, quantity) in value.iter() {
                        writer.append(EventData::Mint {
                            policy: policy.to_hex(),
                            asset: asset.to_hex(),
                            quantity: *quantity,
                        });
                    }
                }
            }

            if let Some(certs) = &tx.certificates {
                for cert in certs.iter() {
                    cert.write_events(&mut writer);
                }
            }

            if let Some(aux) = self.auxiliary_data_set.get(&(idx as u32)) {
                aux.write_events(&mut writer);
            };

            if let Some(witness) = self.transaction_witness_sets.get(idx) {
                if let Some(scripts) = &witness.plutus_script {
                    for script in scripts.iter() {
                        writer.append(EventData::PlutusScriptRef {
                            data: script.to_hex(),
                        });
                    }
                }
            }

            for (idx, input) in tx.inputs.iter().enumerate() {
                let mut writer = writer.child_writer(EventContext {
                    input_idx: Some(idx),
                    ..EventContext::default()
                });

                writer.append(EventData::TxInput {
                    tx_id: input.transaction_id.to_hex(),
                    index: input.index,
                });
            }

            for (idx, output) in tx.outputs.iter().enumerate() {
                let mut writer = writer.child_writer(EventContext {
                    input_idx: Some(idx),
                    ..EventContext::default()
                });

                output.write_events(&mut writer);
            }
        }
    }
}
