use std::ops::Deref;

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
    pub fn new<'b>(storage: &'b mut Storage) -> EventWriter<'b> {
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

    fn child_writer<'b>(&'b mut self, mut extra_context: EventContext) -> EventWriter<'b> {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            storage: &mut self.storage,
        }
    }
}

pub trait EventSource {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter);
}

impl EventSource for Certificate {
    fn write_events(&self, writer: &mut EventWriter) {
        let event = match self {
            Certificate::StakeRegistration(_) => EventData::StakeRegistration,
            Certificate::StakeDeregistration(_) => EventData::StakeDeregistration,
            Certificate::StakeDelegation(_, _) => EventData::StakeDelegation,
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
            } => EventData::PoolRegistration,
            Certificate::PoolRetirement(_, _) => EventData::PoolRetirement,
            Certificate::GenesisKeyDelegation(_, _, _) => EventData::GenesisKeyDelegation,
            Certificate::MoveInstantaneousRewardsCert(_) => EventData::MoveInstantaneousRewardsCert,
        };

        writer.append(event);
    }
}

impl EventSource for Metadatum {
    fn write_events(&self, writer: &mut EventWriter) {
        let key = match self {
            Metadatum::Int(x) => x.to_string(),
            Metadatum::Bytes(x) => hex::encode::<&Vec<u8>>(x.as_ref()),
            Metadatum::Text(x) => x.to_owned(),
            Metadatum::Array(_) => "array".to_string(),
            Metadatum::Map(_) => "map".to_string(),
        };

        writer.append(EventData::Metadata { key });
    }
}

impl EventSource for AuxiliaryData {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        match self {
            AuxiliaryData::Alonzo(data) => {
                if let Some(metadata) = &data.metadata {
                    for (key, _) in metadata {
                        key.write_events(writer);
                    }
                }

                for native in data.native_scripts.iter() {
                    writer.append(EventData::NativeScript);
                }

                for plutus in data.plutus_scripts.iter() {
                    writer.append(EventData::PlutusScript);
                }
            }
            AuxiliaryData::Shelley(data) => {
                for (key, _) in data.iter() {
                    key.write_events(writer);
                }
            }
            _ => log::warn!("ShelleyMa auxiliary data, not sure what to do"),
        }
    }
}

impl EventSource for TransactionOutput {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        writer.append(EventData::TxOutput {
            address: self.address.to_hex(),
            amount: self.amount.clone(),
        });

        match &self.amount {
            Value::Multiasset(coin, assets) => {
                for (policy, assets) in assets.iter() {
                    for (asset, value) in assets.iter() {
                        writer.append(EventData::OutputAsset {
                            policy: policy.to_hex(),
                            asset: asset.to_hex(),
                            value: *value,
                            coin: *coin,
                        });
                    }
                }
            }
            _ => (),
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
