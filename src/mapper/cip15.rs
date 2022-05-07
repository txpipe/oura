use super::EventWriter;
use crate::model::CIP15AssetRecord;
use crate::Error;
use serde_json::Value as JsonValue;

use pallas::ledger::primitives::alonzo::Metadatum;

fn extract_json_property<'a>(json: &'a JsonValue, key: &str) -> Result<&'a JsonValue, Error> {
    let result = json
        .as_object()
        .ok_or_else(|| Error::from("invalid metadatum object for CIP15"))?
        .get(key)
        .ok_or_else(|| Error::from("required key not found for CIP15"))?;

    Ok(result)
}

fn extract_json_string_property(json: &JsonValue, key: &str) -> Result<String, Error> {
    let result = extract_json_property(json, key)?
        .as_str()
        .ok_or_else(|| Error::from("invalid value type for CIP15"))?
        .to_string();

    Ok(result)
}

fn extract_json_int_property(json: &JsonValue, key: &str) -> Result<i64, Error> {
    let result = extract_json_property(json, key)?
        .as_i64()
        .ok_or_else(|| Error::from("invalid value type for CIP15"))?;

    Ok(result)
}

impl EventWriter {
    fn to_cip15_asset_record(&self, content: &Metadatum) -> Result<CIP15AssetRecord, Error> {
        let raw_json = self.to_metadatum_json(content)?;

        Ok(CIP15AssetRecord {
            voting_key: extract_json_string_property(&raw_json, "1")?,
            stake_pub: extract_json_string_property(&raw_json, "2")?,
            reward_address: extract_json_string_property(&raw_json, "3")?,
            nonce: extract_json_int_property(&raw_json, "4")?,
            raw_json,
        })
    }

    pub(crate) fn crawl_metadata_label_61284(&self, content: &Metadatum) -> Result<(), Error> {
        match self.to_cip15_asset_record(content) {
            Ok(record) => self.append_from(record)?,
            Err(err) => log::warn!("error parsing CIP15: {:?}", err),
        }

        Ok(())
    }
}
