use super::EventWriter;
use crate::model::{EventData, MetadataRecord, MetadatumRendition};
use crate::Error;
use pallas::ledger::alonzo::Metadatum;
use serde::Serialize;
use serde_json::value::to_value;

#[derive(Debug, Serialize)]
struct Cip15 {
    voting_key: Vec<u8>,
    stake_pub: Vec<u8>,
    reward_address: Vec<u8>,
    nonce: i64,
}

impl EventWriter {
    pub(crate) fn crawl_metadata_cip15(&self, content: &Metadatum) -> Result<(), Error> {
        let mut entries = match content {
            Metadatum::Map(map) => map,
            _ => return Err("Map expected.".into()),
        }
        .iter();
        let voting_key = match entries.next() {
            Some((Metadatum::Int(1), Metadatum::Bytes(x))) => x.to_vec(),
            _ => return Err("Voting key required".into()),
        };
        let stake_pub = match entries.next() {
            Some((Metadatum::Int(2), Metadatum::Bytes(x))) => x.to_vec(),
            _ => return Err("Stake pub required".into()),
        };
        let reward_address = match entries.next() {
            Some((Metadatum::Int(3), Metadatum::Bytes(x))) => x.to_vec(),
            _ => return Err("Reward address required".into()),
        };
        let nonce = match entries.next() {
            Some(&(Metadatum::Int(4), Metadatum::Int(x))) => x,
            _ => return Err("Integer nonce required".into()),
        };
        self.append(EventData::Metadata(MetadataRecord {
            label: "TEST".to_string(),
            content: MetadatumRendition::MapJson(to_value(Cip15 {
                voting_key,
                stake_pub,
                reward_address,
                nonce,
            })?),
        }))?;
        Ok(())
    }
}
