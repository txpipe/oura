use pallas::ledger::alonzo::Metadatum;

use crate::framework::{CIP25AssetRecord, Error};

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
        _version: &str,
        _policy: &str,
        _asset: &str,
        _content: &Metadatum,
    ) -> Result<CIP25AssetRecord, Error> {
        todo!()
    }

    fn crawl_721_policy(
        &self,
        version: &str,
        policy: &str,
        content: &Metadatum,
    ) -> Result<(), Error> {
        let entries = match content {
            Metadatum::Map(map) => map,
            _ => return Err("expected 721 policy content to be a map".into()),
        };

        for (key, sub_content) in entries.iter() {
            if let Some(asset) = is_asset_key(key) {
                self.to_cip25_asset_record(version, policy, &asset, sub_content)?;
            }
        }

        Ok(())
    }

    pub(crate) fn crawl_metadata_label_721(&self, content: &Metadatum) -> Result<(), Error> {
        let version = self
            .search_cip25_version(content)
            .unwrap_or_else(|| "1.0".to_string());

        let entries = match content {
            Metadatum::Map(map) => map,
            _ => return Err("expected 721 content to be a map".into()),
        };

        for (key, sub_content) in entries.iter() {
            if let Some(policy) = is_policy_key(key) {
                self.crawl_721_policy(&version, &policy, sub_content)?;
            }
        }

        Ok(())
    }
}
