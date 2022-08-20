use pallas::{
    codec::utils::{KeyValuePairs, MaybeIndefArray},
    ledger::primitives::{
        alonzo::{
            AuxiliaryData, Coin, MintedBlock, Multiasset, NativeScript, PlutusData, PlutusScript,
            Redeemer, RewardAccount, TransactionInput, VKeyWitness, Value,
        },
        babbage::{
            LegacyTransactionOutput, PlutusV2Script, PostAlonzoTransactionOutput, TransactionOutput,
        },
        ToHash,
    },
};

use crate::{
    model::{
        MetadataRecord, MintRecord, NativeWitnessRecord, OutputAssetRecord, PlutusDatumRecord,
        PlutusRedeemerRecord, PlutusWitnessRecord, TransactionRecord, TxInputRecord,
        TxOutputRecord, VKeyWitnessRecord, WithdrawalRecord,
    },
    Error,
};

use super::{map::ToHex, EventWriter};

impl EventWriter {
    pub fn collect_input_records(&self, source: &[TransactionInput]) -> Vec<TxInputRecord> {
        source
            .iter()
            .map(|i| self.to_transaction_input_record(i))
            .collect()
    }

    pub fn collect_legacy_output_records(
        &self,
        source: &[LegacyTransactionOutput],
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .iter()
            .map(|i| self.to_legacy_output_record(i))
            .collect()
    }

    pub fn collect_post_alonzo_output_records(
        &self,
        source: &[PostAlonzoTransactionOutput],
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .iter()
            .map(|i| self.to_post_alonzo_output_record(i))
            .collect()
    }

    pub fn collect_any_output_records(
        &self,
        source: &[TransactionOutput],
    ) -> Result<Vec<TxOutputRecord>, Error> {
        source
            .iter()
            .map(|x| match x {
                TransactionOutput::Legacy(x) => self.to_legacy_output_record(x),
                TransactionOutput::PostAlonzo(x) => self.to_post_alonzo_output_record(x),
            })
            .collect()
    }

    pub fn collect_asset_records(&self, amount: &Value) -> Vec<OutputAssetRecord> {
        match amount {
            Value::Coin(_) => vec![],
            Value::Multiasset(_, policies) => policies
                .iter()
                .flat_map(|(policy, assets)| {
                    assets.iter().map(|(asset, amount)| {
                        self.to_transaction_output_asset_record(policy, asset, amount.into())
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

    pub fn collect_withdrawal_records(
        &self,
        withdrawls: &KeyValuePairs<RewardAccount, Coin>,
    ) -> Vec<WithdrawalRecord> {
        withdrawls
            .iter()
            .map(|(reward_account, coin)| WithdrawalRecord {
                reward_account: {
                    let hex = reward_account.to_hex();
                    hex.strip_prefix("e1").map(|x| x.to_string()).unwrap_or(hex)
                },
                coin: coin.into(),
            })
            .collect()
    }

    pub fn collect_metadata_records(
        &self,
        aux_data: &AuxiliaryData,
    ) -> Result<Vec<MetadataRecord>, Error> {
        let metadata = match aux_data {
            AuxiliaryData::PostAlonzo(data) => data.metadata.as_deref(),
            AuxiliaryData::Shelley(data) => Some(data.as_ref()),
            AuxiliaryData::ShelleyMa(data) => Some(data.transaction_metadata.as_ref()),
        };

        match metadata {
            Some(x) => x
                .iter()
                .map(|(label, content)| self.to_metadata_record(label, content))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_vkey_witness_records(
        &self,
        witness_set: &Option<MaybeIndefArray<VKeyWitness>>,
    ) -> Result<Vec<VKeyWitnessRecord>, Error> {
        match witness_set {
            Some(all) => all.iter().map(|i| self.to_vkey_witness_record(i)).collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_native_witness_records(
        &self,
        witness_set: &Option<MaybeIndefArray<NativeScript>>,
    ) -> Result<Vec<NativeWitnessRecord>, Error> {
        match witness_set {
            Some(all) => all
                .iter()
                .map(|i| self.to_native_witness_record(i))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_plutus_v1_witness_records(
        &self,
        witness_set: &Option<MaybeIndefArray<PlutusScript>>,
    ) -> Result<Vec<PlutusWitnessRecord>, Error> {
        match &witness_set {
            Some(all) => all
                .iter()
                .map(|i| self.to_plutus_v1_witness_record(i))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_plutus_v2_witness_records(
        &self,
        witness_set: &Option<MaybeIndefArray<PlutusV2Script>>,
    ) -> Result<Vec<PlutusWitnessRecord>, Error> {
        match &witness_set {
            Some(all) => all
                .iter()
                .map(|i| self.to_plutus_v2_witness_record(i))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_plutus_redeemer_records(
        &self,
        witness_set: &Option<MaybeIndefArray<Redeemer>>,
    ) -> Result<Vec<PlutusRedeemerRecord>, Error> {
        match &witness_set {
            Some(all) => all
                .iter()
                .map(|i| self.to_plutus_redeemer_record(i))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_plutus_datum_records(
        &self,
        witness_set: &Option<MaybeIndefArray<PlutusData>>,
    ) -> Result<Vec<PlutusDatumRecord>, Error> {
        match &witness_set {
            Some(all) => all.iter().map(|i| self.to_plutus_datum_record(i)).collect(),
            None => Ok(vec![]),
        }
    }

    pub fn collect_shelley_tx_records(
        &self,
        block: &MintedBlock,
    ) -> Result<Vec<TransactionRecord>, Error> {
        block
            .transaction_bodies
            .iter()
            .enumerate()
            .map(|(idx, tx)| {
                let aux_data = block
                    .auxiliary_data_set
                    .iter()
                    .find(|(k, _)| *k == (idx as u32))
                    .map(|(_, v)| v);

                let witness_set = block.transaction_witness_sets.get(idx);

                let tx_hash = tx.to_hash().to_hex();

                self.to_transaction_record(tx, &tx_hash, aux_data, witness_set)
            })
            .collect()
    }
}
