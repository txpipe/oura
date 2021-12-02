use std::ops::Deref;

use pallas::ledger::alonzo::{AuxiliaryData, Block, Certificate, Metadatum};

use crate::ports::Event;

pub type Storage = Vec<(Option<Event>, Event)>;

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
    parent: Option<Event>,
    storage: &'a mut Storage,
}

impl<'a> EventWriter<'a> {
    pub fn new<'b>(storage: &'b mut Storage) -> EventWriter<'b> {
        EventWriter {
            parent: None,
            // we initialize our vec with an initial heuristic-based capacity
            storage,
        }
    }

    fn append(&mut self, event: Event) -> &mut Self {
        match &self.parent {
            Some(parent) => self.storage.push((Some(parent.clone()), event)),
            None => self.storage.push((None, event)),
        }

        self
    }

    fn child_writer<'b>(&'b mut self, parent: Event) -> EventWriter<'b> {
        EventWriter {
            parent: Some(parent),
            storage: &mut self.storage,
        }
    }

    fn start_append<'b>(&'b mut self, event: Event) -> EventWriter<'b> {
        self.append(event.clone());
        self.child_writer(event)
    }

    pub fn print(&self) {
        self.storage.iter().for_each(|e| println!("[{:#?}]", e));
    }
}

pub trait EventSource {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter);
}

impl EventSource for Certificate {
    fn write_events(&self, writer: &mut EventWriter) {
        let event = match self {
            Certificate::StakeRegistration(_) => Event::StakeRegistration,
            Certificate::StakeDeregistration(_) => Event::StakeDeregistration,
            Certificate::StakeDelegation(_, _) => Event::StakeDelegation,
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
            } => Event::PoolRegistration,
            Certificate::PoolRetirement(_, _) => Event::PoolRetirement,
            Certificate::GenesisKeyDelegation(_, _, _) => Event::GenesisKeyDelegation,
            Certificate::MoveInstantaneousRewardsCert(_) => Event::MoveInstantaneousRewardsCert,
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

        writer.append(Event::Metadata { key });
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
                    writer.append(Event::NativeScript);
                }

                for plutus in data.plutus_scripts.iter() {
                    writer.append(Event::PlutusScript);
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

impl EventSource for Block {
    fn write_events<'a>(&'a self, writer: &'a mut EventWriter) {
        // we add an event signalling the begining of a block
        let evt = Event::Block {
            block_number: self.header.header_body.block_number,
            slot: self.header.header_body.slot,
        };

        let mut writer = writer.start_append(evt);

        for (idx, tx) in self.transaction_bodies.iter().enumerate() {
            let mut writer = writer.start_append(Event::Transaction {
                fee: tx.fee,
                ttl: tx.ttl,
                validity_interval_start: tx.validity_interval_start,
            });

            if let Some(mint) = &tx.mint {
                for (key1, value) in mint.iter() {
                    for (key2, quantity) in value.iter() {
                        writer.append(Event::Mint {
                            key1: key1.to_hex(),
                            key2: key2.to_hex(),
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

            for tx_input in tx.inputs.iter() {
                writer.append(Event::TxInput {
                    transaction_id: tx_input.transaction_id.to_hex(),
                    index: tx_input.index,
                });
            }

            for tx_output in tx.outputs.iter() {
                writer.append(Event::TxOutput{
                    address: tx_output.address.to_hex(),
                    amount: tx_output.amount.clone(),
                });
            }
        }
    }
}
