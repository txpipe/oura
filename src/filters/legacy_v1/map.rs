use lazy_static::__Deref;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

use pallas::ledger::primitives::babbage::{MintedDatumOption, NetworkId};
use pallas::ledger::primitives::{
    alonzo::{
        self as alonzo, Certificate, InstantaneousRewardSource, InstantaneousRewardTarget,
        Metadatum, MetadatumLabel, Relay,
    },
    babbage, ToCanonicalJson,
};
use pallas::ledger::traverse::{
    ComputeHash, MultiEraAsset, MultiEraBlock, MultiEraCert, MultiEraInput, MultiEraOutput,
    MultiEraTx, OriginalHash,
};
use pallas::network::miniprotocols::Point;
use pallas::{codec::utils::KeepRaw, crypto::hash::Hash};

use crate::framework::legacy_v1::*;
use crate::framework::AppliesPolicy;
use crate::framework::Error as OuraError;

use super::EventWriter;

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Vec<u8> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl ToHex for &[u8] {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl<const BYTES: usize> ToHex for Hash<BYTES> {
    fn to_hex(&self) -> String {
        hex::encode(self)
    }
}

impl From<&alonzo::StakeCredential> for StakeCredential {
    fn from(other: &alonzo::StakeCredential) -> Self {
        match other {
            alonzo::StakeCredential::AddrKeyhash(x) => StakeCredential::AddrKeyhash(x.to_hex()),
            alonzo::StakeCredential::Scripthash(x) => StakeCredential::Scripthash(x.to_hex()),
        }
    }
}

fn ip_string_from_bytes(bytes: &[u8]) -> String {
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

impl From<Point> for EventData {
    fn from(value: Point) -> Self {
        match value {
            Point::Origin => EventData::RollBack {
                block_slot: 0,
                block_hash: "".to_string(),
            },
            Point::Specific(slot, hash) => EventData::RollBack {
                block_slot: slot,
                block_hash: hex::encode(hash),
            },
        }
    }
}

impl From<&MultiEraInput<'_>> for TxInputRecord {
    fn from(value: &MultiEraInput) -> Self {
        Self {
            tx_id: value.hash().to_string(),
            index: value.index(),
        }
    }
}

impl From<&MultiEraAsset<'_>> for OutputAssetRecord {
    fn from(value: &MultiEraAsset<'_>) -> Self {
        Self {
            policy: value.policy().map(ToString::to_string).unwrap_or_default(),
            asset: value.name().map(|x| x.to_hex()).unwrap_or_default(),
            asset_ascii: value.to_ascii_name(),
            amount: value.coin() as u64,
        }
    }
}

impl From<&KeepRaw<'_, alonzo::PlutusData>> for PlutusDatumRecord {
    fn from(value: &KeepRaw<'_, alonzo::PlutusData>) -> Self {
        Self {
            datum_hash: value.original_hash().to_hex(),
            plutus_data: value.to_json(),
        }
    }
}

fn relay_to_string(relay: &Relay) -> String {
    match relay {
        Relay::SingleHostAddr(port, ipv4, ipv6) => {
            let ip = match (ipv6, ipv4) {
                (None, None) => "".to_string(),
                (_, Some(x)) => ip_string_from_bytes(x.as_ref()),
                (Some(x), _) => ip_string_from_bytes(x.as_ref()),
            };

            match port {
                Some(port) => format!("{ip}:{port}"),
                None => ip,
            }
        }
        Relay::SingleHostName(port, host) => match port {
            Some(port) => format!("{host}:{port}"),
            None => host.clone(),
        },
        Relay::MultiHostName(host) => host.clone(),
    }
}

fn metadatum_to_string_key(datum: &Metadatum) -> String {
    match datum {
        Metadatum::Int(x) => x.to_string(),
        Metadatum::Bytes(x) => hex::encode(x.as_slice()),
        Metadatum::Text(x) => x.to_owned(),
        x => {
            log::warn!("unexpected metadatum type for label: {:?}", x);
            Default::default()
        }
    }
}

impl EventWriter<'_> {
    pub fn to_transaction_output_record(&self, output: &MultiEraOutput) -> TxOutputRecord {
        let address = output
            .address()
            .map_err(OuraError::parse)
            .apply_policy(self.error_policy)
            .unwrap();

        TxOutputRecord {
            address: address.map(|x| x.to_string()).unwrap_or_default(),
            amount: output.lovelace_amount(),
            assets: output
                .non_ada_assets()
                .iter()
                .map(|x| OutputAssetRecord::from(x))
                .collect::<Vec<_>>()
                .into(),
            datum_hash: match &output.datum() {
                Some(MintedDatumOption::Hash(x)) => Some(x.to_string()),
                Some(MintedDatumOption::Data(x)) => Some(x.original_hash().to_hex()),
                None => None,
            },
            inline_datum: match &output.datum() {
                Some(MintedDatumOption::Data(x)) => Some(PlutusDatumRecord::from(x.deref())),
                _ => None,
            },
        }
    }
    pub fn to_withdrawal_record(&self, withdrawal: (&[u8], u64)) -> WithdrawalRecord {
        WithdrawalRecord {
            reward_account: {
                let hex = withdrawal.0.to_hex();
                hex.strip_prefix("e1").map(|x| x.to_string()).unwrap_or(hex)
            },
            coin: withdrawal.1,
        }
    }

    pub fn to_transaction_record(&self, tx: &MultiEraTx) -> TransactionRecord {
        let mut record = TransactionRecord {
            hash: tx.hash().to_string(),
            size: tx.size() as u32,
            fee: tx.fee().unwrap_or_default(),
            ttl: tx.ttl(),
            validity_interval_start: tx.validity_start(),
            network_id: tx.network_id().map(|x| match x {
                NetworkId::One => 1,
                NetworkId::Two => 2,
            }),
            ..Default::default()
        };

        let outputs: Vec<_> = tx
            .outputs()
            .iter()
            .map(|x| self.to_transaction_output_record(x))
            .collect();

        record.output_count = outputs.len();
        record.total_output = outputs.iter().map(|o| o.amount).sum();

        let inputs: Vec<_> = tx.inputs().iter().map(|x| TxInputRecord::from(x)).collect();

        record.input_count = inputs.len();

        let mints: Vec<_> = tx.mints().iter().map(|x| self.to_mint_record(x)).collect();

        record.mint_count = mints.len();

        let collateral_inputs: Vec<_> = tx
            .collateral()
            .iter()
            .map(|x| TxInputRecord::from(x))
            .collect();

        record.collateral_input_count = collateral_inputs.len();

        let collateral_return = tx.collateral_return();

        record.has_collateral_output = collateral_return.is_some();

        // TODO
        // TransactionBodyComponent::ScriptDataHash(_)
        // TransactionBodyComponent::RequiredSigners(_)
        // TransactionBodyComponent::AuxiliaryDataHash(_)

        if self.config.include_transaction_details {
            record.outputs = Some(outputs);
            record.inputs = Some(inputs);
            record.mint = Some(mints);

            record.collateral_inputs = Some(collateral_inputs);

            record.collateral_output =
                collateral_return.map(|x| self.to_transaction_output_record(&x));

            record.metadata = tx
                .metadata()
                .collect::<Vec<_>>()
                .iter()
                .map(|(l, v)| self.to_metadata_record(l, v))
                .collect::<Vec<_>>()
                .into();

            record.vkey_witnesses = tx
                .vkey_witnesses()
                .iter()
                .map(|x| self.to_vkey_witness_record(x))
                .collect::<Vec<_>>()
                .into();

            record.native_witnesses = tx
                .native_scripts()
                .iter()
                .map(|x| self.to_native_witness_record(x))
                .collect::<Vec<_>>()
                .into();

            let v1_scripts = tx
                .plutus_v1_scripts()
                .iter()
                .map(|x| self.to_plutus_v1_witness_record(x))
                .collect::<Vec<_>>();

            let v2_scripts = tx
                .plutus_v2_scripts()
                .iter()
                .map(|x| self.to_plutus_v2_witness_record(x))
                .collect::<Vec<_>>();

            record.plutus_witnesses = Some([v1_scripts, v2_scripts].concat());

            record.plutus_redeemers = tx
                .redeemers()
                .iter()
                .map(|x| self.to_plutus_redeemer_record(x))
                .collect::<Vec<_>>()
                .into();

            record.plutus_data = tx
                .plutus_data()
                .iter()
                .map(|x| PlutusDatumRecord::from(x))
                .collect::<Vec<_>>()
                .into();

            record.withdrawals = tx
                .withdrawals()
                .collect::<Vec<_>>()
                .iter()
                .map(|x| self.to_withdrawal_record(*x))
                .collect::<Vec<_>>()
                .into();
        }

        record
    }

    pub fn to_block_record(&self, source: &MultiEraBlock, cbor: &[u8]) -> BlockRecord {
        let header = source.header();
        let (epoch, sub_slot) = source.epoch(self.genesis);

        let mut record = BlockRecord {
            era: source.era().into(),
            body_size: source.body_size().unwrap_or_default(),
            issuer_vkey: header.issuer_vkey().map(hex::encode).unwrap_or_default(),
            vrf_vkey: header.vrf_vkey().map(hex::encode).unwrap_or_default(),
            tx_count: source.tx_count(),
            hash: source.hash().to_string(),
            number: source.number(),
            slot: source.slot(),
            epoch: Some(epoch),
            epoch_slot: Some(sub_slot),
            previous_hash: header
                .previous_hash()
                .map(|x| x.to_string())
                .unwrap_or_default(),
            cbor_hex: match self.config.include_block_cbor {
                true => Some(hex::encode(cbor)),
                false => None,
            },
            transactions: None,
        };

        if self.config.include_block_details {
            let txs = source
                .txs()
                .iter()
                .map(|x| self.to_transaction_record(x))
                .collect();

            record.transactions = Some(txs);
        }

        record
    }

    pub fn to_mint_record(&self, asset: &MultiEraAsset) -> MintRecord {
        MintRecord {
            policy: asset.policy().map(|x| x.to_string()).unwrap_or_default(),
            asset: asset.name().map(hex::encode).unwrap_or_default(),
            quantity: asset.coin(),
        }
    }

    pub fn to_metadatum_json_map_entry(
        &self,
        pair: (&Metadatum, &Metadatum),
    ) -> (String, JsonValue) {
        let key = metadatum_to_string_key(pair.0);
        let value = self.to_metadatum_json(pair.1);
        (key, value)
    }

    pub fn to_metadatum_json(&self, source: &Metadatum) -> JsonValue {
        match source {
            Metadatum::Int(x) => json!(i128::from(*x)),
            Metadatum::Bytes(x) => json!(hex::encode(x.as_slice())),
            Metadatum::Text(x) => json!(x),
            Metadatum::Array(x) => {
                let items: Vec<_> = x.iter().map(|x| self.to_metadatum_json(x)).collect();

                json!(items)
            }
            Metadatum::Map(x) => {
                let map: HashMap<_, _> = x
                    .iter()
                    .map(|(key, value)| self.to_metadatum_json_map_entry((key, value)))
                    .collect();

                json!(map)
            }
        }
    }

    pub fn to_metadata_record(&self, label: &MetadatumLabel, value: &Metadatum) -> MetadataRecord {
        MetadataRecord {
            label: label.to_string(),
            content: match value {
                Metadatum::Int(x) => MetadatumRendition::IntScalar(i128::from(*x)),
                Metadatum::Bytes(x) => MetadatumRendition::BytesHex(hex::encode(x.as_slice())),
                Metadatum::Text(x) => MetadatumRendition::TextScalar(x.clone()),
                Metadatum::Array(_) => MetadatumRendition::ArrayJson(self.to_metadatum_json(value)),
                Metadatum::Map(_) => MetadatumRendition::MapJson(self.to_metadatum_json(value)),
            },
        }
    }

    pub fn to_aux_native_script_event(&self, script: &alonzo::NativeScript) -> EventData {
        EventData::NativeScript {
            policy_id: script.compute_hash().to_hex(),
            script: script.to_json(),
        }
    }

    pub fn to_aux_plutus_script_event(&self, script: &alonzo::PlutusScript) -> EventData {
        EventData::PlutusScript {
            hash: script.compute_hash().to_hex(),
            data: script.0.to_hex(),
        }
    }

    pub fn to_plutus_redeemer_record(&self, redeemer: &alonzo::Redeemer) -> PlutusRedeemerRecord {
        PlutusRedeemerRecord {
            purpose: match redeemer.tag {
                alonzo::RedeemerTag::Spend => "spend".to_string(),
                alonzo::RedeemerTag::Mint => "mint".to_string(),
                alonzo::RedeemerTag::Cert => "cert".to_string(),
                alonzo::RedeemerTag::Reward => "reward".to_string(),
            },
            ex_units_mem: redeemer.ex_units.mem,
            ex_units_steps: redeemer.ex_units.steps,
            input_idx: redeemer.index,
            plutus_data: redeemer.data.to_json(),
        }
    }

    pub fn to_plutus_v1_witness_record(
        &self,
        script: &alonzo::PlutusScript,
    ) -> PlutusWitnessRecord {
        PlutusWitnessRecord {
            script_hash: script.compute_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        }
    }

    pub fn to_plutus_v2_witness_record(
        &self,
        script: &babbage::PlutusV2Script,
    ) -> PlutusWitnessRecord {
        PlutusWitnessRecord {
            script_hash: script.compute_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        }
    }

    pub fn to_native_witness_record(&self, script: &alonzo::NativeScript) -> NativeWitnessRecord {
        NativeWitnessRecord {
            policy_id: script.compute_hash().to_hex(),
            script_json: script.to_json(),
        }
    }

    pub fn to_vkey_witness_record(&self, witness: &alonzo::VKeyWitness) -> VKeyWitnessRecord {
        VKeyWitnessRecord {
            vkey_hex: witness.vkey.to_hex(),
            signature_hex: witness.signature.to_hex(),
        }
    }

    pub fn to_certificate_event(&self, cert: &MultiEraCert) -> Option<EventData> {
        let evt = match cert.as_alonzo()? {
            Certificate::StakeRegistration(credential) => EventData::StakeRegistration {
                credential: credential.into(),
            },
            Certificate::StakeDeregistration(credential) => EventData::StakeDeregistration {
                credential: credential.into(),
            },
            Certificate::StakeDelegation(credential, pool) => EventData::StakeDelegation {
                credential: credential.into(),
                pool_hash: pool.to_hex(),
            },
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
            } => EventData::PoolRegistration {
                operator: operator.to_hex(),
                vrf_keyhash: vrf_keyhash.to_hex(),
                pledge: *pledge,
                cost: *cost,
                margin: (margin.numerator as f64 / margin.denominator as f64),
                reward_account: reward_account.to_hex(),
                pool_owners: pool_owners.iter().map(|p| p.to_hex()).collect(),
                relays: relays.iter().map(relay_to_string).collect(),
                pool_metadata: pool_metadata.as_ref().map(|m| m.url.clone()),
                pool_metadata_hash: pool_metadata.as_ref().map(|m| m.hash.clone().to_hex()),
            },
            Certificate::PoolRetirement(pool, epoch) => EventData::PoolRetirement {
                pool: pool.to_hex(),
                epoch: *epoch,
            },
            Certificate::MoveInstantaneousRewardsCert(move_) => {
                EventData::MoveInstantaneousRewardsCert {
                    from_reserves: matches!(move_.source, InstantaneousRewardSource::Reserves),
                    from_treasury: matches!(move_.source, InstantaneousRewardSource::Treasury),
                    to_stake_credentials: match &move_.target {
                        InstantaneousRewardTarget::StakeCredentials(creds) => {
                            let x = creds.iter().map(|(k, v)| (k.into(), *v)).collect();
                            Some(x)
                        }
                        _ => None,
                    },
                    to_other_pot: match move_.target {
                        InstantaneousRewardTarget::OtherAccountingPot(x) => Some(x),
                        _ => None,
                    },
                }
            }
            // TODO: not likely, leaving for later
            Certificate::GenesisKeyDelegation(..) => EventData::GenesisKeyDelegation {},
        };

        Some(evt)
    }

    pub fn to_collateral_event(&self, collateral: &MultiEraInput) -> EventData {
        EventData::Collateral {
            tx_id: collateral.hash().to_string(),
            index: collateral.index(),
        }
    }
}
