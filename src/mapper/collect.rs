use pallas::ledger::primitives::alonzo::{
    AuxiliaryData, Multiasset, TransactionInput, TransactionOutput, Value,
};

use crate::{
    model::{MetadataRecord, MintRecord, OutputAssetRecord, TxInputRecord, TxOutputRecord},
    Error,
};

use super::EventWriter;

impl EventWriter {
    pub fn collect_input_records(&self, source: &[TransactionInput]) -> Vec<TxInputRecord> {
        source
            .iter()
            .map(|i| self.to_transaction_input_record(i))
            .collect()
    }

    pub fn collect_output_records(
        &self,
        source: &[TransactionOutput],
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .iter()
            .map(|i| self.to_transaction_output_record(i))
            .collect()
    }

    pub fn collect_asset_records(&self, amount: &Value) -> Vec<OutputAssetRecord> {
        match amount {
            Value::Coin(_) => vec![],
            Value::Multiasset(_, policies) => policies
                .iter()
                .flat_map(|(policy, assets)| {
                    assets.iter().map(|(asset, amount)| {
                        self.to_transaction_output_asset_record(policy, asset, *amount)
                    })
                })
                .collect(),
        }
    }

    pub fn collect_mint_records(&self, mint: &Multiasset<i64>) -> Vec<MintRecord> {
        mint.iter()
            .flat_map(|(policy, assets)| {
                assets
                    .iter()
                    .map(|(asset, amount)| self.to_mint_record(policy, asset, *amount))
            })
            .collect()
    }

    pub fn collect_metadata_records(
        &self,
        aux_data: &AuxiliaryData,
    ) -> Result<Vec<MetadataRecord>, Error> {
        let metadata = match aux_data {
            AuxiliaryData::Alonzo(data) => data.metadata.as_deref(),
            AuxiliaryData::Shelley(data) => Some(data.as_ref()),
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                ..
            } => Some(transaction_metadata.as_ref()),
        };

        match metadata {
            Some(x) => x
                .iter()
                .map(|(label, content)| self.to_metadata_record(label, content))
                .collect(),
            None => Ok(vec![]),
        }
    }
}
