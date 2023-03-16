use serde_json::Value as JsonValue;

use pallas::ledger::primitives::alonzo::Metadatum;

use super::EventWriter;
use crate::framework::legacy_v1::*;

fn extract_json_property<'a>(json: &'a JsonValue, key: &str) -> Option<&'a JsonValue> {
    json.as_object().and_then(|x| x.get(key))
}

fn extract_json_string_property(json: &JsonValue, key: &str) -> Option<String> {
    extract_json_property(json, key)
        .and_then(|x| x.as_str())
        .map(|x| x.to_owned())
}

fn extract_json_int_property(json: &JsonValue, key: &str) -> Option<i64> {
    extract_json_property(json, key).and_then(|x| x.as_i64())
}

impl EventWriter<'_> {
    fn to_cip15_asset_record(&self, content: &Metadatum) -> CIP15AssetRecord {
        let raw_json = self.to_metadatum_json(content);

        CIP15AssetRecord {
            voting_key: extract_json_string_property(&raw_json, "1").unwrap_or_default(),
            stake_pub: extract_json_string_property(&raw_json, "2").unwrap_or_default(),
            reward_address: extract_json_string_property(&raw_json, "3").unwrap_or_default(),
            nonce: extract_json_int_property(&raw_json, "4").unwrap_or_default(),
            raw_json,
        }
    }

    pub(crate) fn crawl_metadata_label_61284(
        &mut self,
        content: &Metadatum,
    ) -> Result<(), gasket::error::Error> {
        self.append_from(self.to_cip15_asset_record(content))?;

        Ok(())
    }
}
