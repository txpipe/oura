use serde_json::Value as JsonValue;

use pallas::ledger::primitives::alonzo::Metadatum;

use crate::{model::CIP25AssetRecord, Error};

use super::EventWriter;

// Heuristic approach for filtering policy entries. This is the best I could
// come up with. Is there a better, official way?
fn is_policy_key(key: &Metadatum) -> Option<String> {
    match key {
        Metadatum::Bytes(x) if x.len() == 28 => Some(hex::encode(x.as_slice())),
        Metadatum::Text(x) if x.len() == 56 => Some(x.to_owned()),
        _ => None,
    }
}

// Heuristic approach for filtering asset entries. Even less strict than when
// searching for policies. In this case, we only check for valid data types.
// There's probably a much more formal approach.
fn is_asset_key(key: &Metadatum) -> Option<String> {
    match key {
        Metadatum::Bytes(x) => Some(hex::encode(x.as_slice())),
        Metadatum::Text(x) => Some(x.to_owned()),
        _ => None,
    }
}

fn extract_json_property(json: &JsonValue, key: &str) -> Option<String> {
    json.as_object()
        .and_then(|obj| obj.get(key))
        .and_then(|v| v.as_str())
        .map(|x| x.to_string())
}

impl EventWriter {
    fn search_cip25_version(&self, content_721: &Metadatum) -> Option<String> {
        match content_721 {
            Metadatum::Map(entries) => entries.iter().find_map(|(key, value)| match key {
                Metadatum::Text(x) if x == "version" => match value {
                    Metadatum::Text(value) => Some(value.to_owned()),
                    _ => None,
                },
                _ => None,
            }),
            _ => None,
        }
    }

    fn to_cip25_asset_record(
        &self,
        version: &str,
        policy: &str,
        asset: &str,
        content: &Metadatum,
    ) -> Result<CIP25AssetRecord, Error> {
        let raw_json = self.to_metadatum_json(content)?;

        Ok(CIP25AssetRecord {
            policy: policy.to_string(),
            asset: asset.to_string(),
            version: version.to_string(),
            name: extract_json_property(&raw_json, "name"),
            media_type: extract_json_property(&raw_json, "mediaType"),
            image: extract_json_property(&raw_json, "image"),
            description: extract_json_property(&raw_json, "description"),
            raw_json,
        })
    }

    fn crawl_721_policy(
        &self,
        version: &str,
        policy: &str,
        content: &Metadatum,
    ) -> Result<(), Error> {
        if let Metadatum::Map(entries) = content {
            for (key, sub_content) in entries.iter() {
                if let Some(asset) = is_asset_key(key) {
                    let record =
                        self.to_cip25_asset_record(version, policy, &asset, sub_content)?;
                    self.append_from(record)?;
                }
            }
        } else {
            log::warn!("invalid metadatum type for policy inside 721 label");
        }

        Ok(())
    }

    pub(crate) fn crawl_metadata_label_721(&self, content: &Metadatum) -> Result<(), Error> {
        let version = self
            .search_cip25_version(content)
            .unwrap_or_else(|| "1.0".to_string());

        if let Metadatum::Map(entries) = content {
            for (key, sub_content) in entries.iter() {
                if let Some(policy) = is_policy_key(key) {
                    self.crawl_721_policy(&version, &policy, sub_content)?;
                }
            }
        } else {
            log::warn!("invalid metadatum type for 721 label");
        }

        Ok(())
    }
}
