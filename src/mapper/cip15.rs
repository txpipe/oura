use super::EventWriter;
use crate::model::CIP15AssetRecord;
use crate::Error;
use serde_json::Value as JsonValue;

use pallas::ledger::primitives::alonzo::Metadatum;

fn extract_json_property<'a>(
    json: &'a JsonValue,
    key: &str,
) -> Result<Option<&'a JsonValue>, Error> {
    let result = json
        .as_object()
        .ok_or_else(|| Error::from("invalid metadatum object for CIP15"))?
        .get(key);

    Ok(result)
}

fn extract_json_string_property(json: &JsonValue, key: &str) -> Result<Option<String>, Error> {
    let result = extract_json_property(json, key)?
        .and_then(|x| x.as_str())
        .map(|x| x.to_string());

    Ok(result)
}

fn extract_json_int_property(json: &JsonValue, key: &str) -> Result<Option<i64>, Error> {
    let result = extract_json_property(json, key)?.and_then(|x| x.as_i64());

    Ok(result)
}

impl EventWriter {
    fn to_cip15_asset_record(&self, content: &Metadatum) -> Result<CIP15AssetRecord, Error> {
        let raw_json = self.to_metadatum_json(content)?;

        Ok(CIP15AssetRecord {
            voting_key: extract_json_string_property(&raw_json, "1")?
                .ok_or_else(|| Error::from("invalid value type for CIP15"))?,
            stake_pub: extract_json_string_property(&raw_json, "2")?
                .ok_or_else(|| Error::from("invalid value type for CIP15"))?,
            reward_address: extract_json_string_property(&raw_json, "3")?.unwrap_or_default(),
            nonce: extract_json_int_property(&raw_json, "4")?.unwrap_or_default(),
            raw_json,
        })
    }

    pub(crate) fn crawl_metadata_label_61284(&self, content: &Metadatum) -> Result<(), Error> {
        match self.to_cip15_asset_record(content) {
            Ok(record) => self.append_from(record)?,
            Err(err) => log::info!("error parsing CIP15: {:?}", err),
        }

        Ok(())
    }
}
