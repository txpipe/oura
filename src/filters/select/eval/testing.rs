use utxorpc_spec::utxorpc::v1alpha::cardano::{metadatum, AuxData, Metadata, Metadatum, Tx};

use super::*;

pub fn multiasset_combo(policy_hex: &str, asset_prefix: &str) -> Multiasset {
    Multiasset {
        policy_id: hex::decode(policy_hex).unwrap().into(),
        assets: vec![
            Asset {
                name: format!("{asset_prefix}1").as_bytes().to_vec().into(),
                output_coin: 345000000,
                mint_coin: 0,
            },
            Asset {
                name: format!("{asset_prefix}2").as_bytes().to_vec().into(),
                output_coin: 345000000,
                mint_coin: 0,
            },
        ],
    }
}

pub fn metadata_combo(label: u64, text: &str) -> Metadata {
    Metadata {
        label,
        value: Metadatum {
            metadatum: metadatum::Metadatum::Text(text.into()).into(),
        }
        .into(),
    }
}

pub fn test_vectors() -> Vec<Tx> {
    let tx0 = Tx::default();

    let tx1 = Tx {
        outputs: vec![TxOutput {
            address: hex::decode("019493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251").unwrap().into(),
            coin: 123000000,
            assets: vec![
                multiasset_combo("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373", "abc"),
                multiasset_combo("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209", "123")
            ],
            datum_hash: hex::decode("923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec").unwrap().into(),
            ..Default::default()
        }],
        auxiliary: Some(AuxData {
            metadata: vec![
                metadata_combo(127, "lorem"),
                metadata_combo(9980, "ipsum")
            ],
            ..Default::default()
        }),
        ..Default::default()
    };

    let tx2 = Tx {
        outputs: vec![TxOutput {
            address: hex::decode("619493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e")
                .unwrap()
                .into(),
            coin: 123000000,
            assets: vec![multiasset_combo(
                "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
                "abc",
            )],
            datum_hash: hex::decode(
                "923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec",
            )
            .unwrap()
            .into(),
            ..Default::default()
        }],
        mint: vec![multiasset_combo(
            "533bb94a8850ee3ccbe483106489399112b74c905342cb1792a797a0",
            "xyz",
        )],
        auxiliary: Some(AuxData {
            metadata: vec![metadata_combo(127, "lorem")],
            ..Default::default()
        }),
        ..Default::default()
    };

    let tx3 = Tx {
        outputs: vec![TxOutput {
            address: hex::decode("019493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251").unwrap().into(),
            coin: 123000000,
            assets: vec![
                multiasset_combo("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209", "123")
            ],
            datum_hash: hex::decode("923918e403bf43c34b4ef6b48eb2ee04babed17320d8d1b9ff9ad086e86f44ec").unwrap().into(),
            ..Default::default()
        }],
        auxiliary: Some(AuxData {
            metadata: vec![
                metadata_combo(9980, "ipsum")
            ],
            ..Default::default()
        }),
        ..Default::default()
    };

    vec![tx0, tx1, tx2, tx3]
}

pub fn find_positive_test_vectors(predicate: impl Into<Predicate>) -> Vec<usize> {
    let subjects = test_vectors();
    let predicate = predicate.into();

    subjects
        .into_iter()
        .enumerate()
        .filter_map(|(idx, subject)| match eval_tx(&subject, &predicate) {
            MatchOutcome::Positive => Some(idx),
            _ => None,
        })
        .collect()
}
