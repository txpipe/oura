use pallas::ledger::traverse::{ComputeHash, MultiEraCert, MultiEraInput, OriginalHash};
use pallas::{codec::utils::KeepRaw, crypto::hash::Hash};
use std::collections::HashMap;

use pallas::ledger::primitives::{
    alonzo::{
        self as alonzo, Certificate, InstantaneousRewardSource, InstantaneousRewardTarget,
        Metadatum, MetadatumLabel, Relay, Value,
    },
    babbage, ToCanonicalJson,
};

use pallas::network::miniprotocols::Point;
use serde_json::{json, Value as JsonValue};

use crate::framework::legacy_v1::*;
use crate::Error;

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

fn get_tx_output_coin_value(amount: &Value) -> u64 {
    match amount {
        Value::Coin(x) => *x,
        Value::Multiasset(x, _) => *x,
    }
}

impl EventWriter {
    pub fn to_metadatum_json_map_entry(
        &self,
        pair: (&Metadatum, &Metadatum),
    ) -> Result<(String, JsonValue), Error> {
        let key = metadatum_to_string_key(pair.0);
        let value = self.to_metadatum_json(pair.1)?;
        Ok((key, value))
    }

    pub fn to_metadatum_json(&self, source: &Metadatum) -> Result<JsonValue, Error> {
        match source {
            Metadatum::Int(x) => Ok(json!(i128::from(*x))),
            Metadatum::Bytes(x) => Ok(json!(hex::encode(x.as_slice()))),
            Metadatum::Text(x) => Ok(json!(x)),
            Metadatum::Array(x) => {
                let items: Result<Vec<_>, _> =
                    x.iter().map(|x| self.to_metadatum_json(x)).collect();

                Ok(json!(items?))
            }
            Metadatum::Map(x) => {
                let map: Result<HashMap<_, _>, _> = x
                    .iter()
                    .map(|(key, value)| self.to_metadatum_json_map_entry((key, value)))
                    .collect();

                Ok(json!(map?))
            }
        }
    }

    pub fn to_metadata_record(
        &self,
        label: &MetadatumLabel,
        value: &Metadatum,
    ) -> Result<MetadataRecord, Error> {
        let data = MetadataRecord {
            label: label.to_string(),
            content: match value {
                Metadatum::Int(x) => MetadatumRendition::IntScalar(i128::from(*x)),
                Metadatum::Bytes(x) => MetadatumRendition::BytesHex(hex::encode(x.as_slice())),
                Metadatum::Text(x) => MetadatumRendition::TextScalar(x.clone()),
                Metadatum::Array(_) => {
                    MetadatumRendition::ArrayJson(self.to_metadatum_json(value)?)
                }
                Metadatum::Map(_) => MetadatumRendition::MapJson(self.to_metadatum_json(value)?),
            },
        };

        Ok(data)
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

    pub fn to_plutus_redeemer_record(
        &self,
        redeemer: &alonzo::Redeemer,
    ) -> Result<PlutusRedeemerRecord, crate::Error> {
        Ok(PlutusRedeemerRecord {
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
        })
    }

    pub fn to_plutus_datum_record(
        &self,
        datum: &KeepRaw<'_, alonzo::PlutusData>,
    ) -> Result<PlutusDatumRecord, crate::Error> {
        Ok(PlutusDatumRecord {
            datum_hash: datum.original_hash().to_hex(),
            plutus_data: datum.to_json(),
        })
    }

    pub fn to_plutus_v1_witness_record(
        &self,
        script: &alonzo::PlutusScript,
    ) -> Result<PlutusWitnessRecord, crate::Error> {
        Ok(PlutusWitnessRecord {
            script_hash: script.compute_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        })
    }

    pub fn to_plutus_v2_witness_record(
        &self,
        script: &babbage::PlutusV2Script,
    ) -> Result<PlutusWitnessRecord, crate::Error> {
        Ok(PlutusWitnessRecord {
            script_hash: script.compute_hash().to_hex(),
            script_hex: script.as_ref().to_hex(),
        })
    }

    pub fn to_native_witness_record(
        &self,
        script: &alonzo::NativeScript,
    ) -> Result<NativeWitnessRecord, crate::Error> {
        Ok(NativeWitnessRecord {
            policy_id: script.compute_hash().to_hex(),
            script_json: script.to_json(),
        })
    }

    pub fn to_vkey_witness_record(
        &self,
        witness: &alonzo::VKeyWitness,
    ) -> Result<VKeyWitnessRecord, crate::Error> {
        Ok(VKeyWitnessRecord {
            vkey_hex: witness.vkey.to_hex(),
            signature_hex: witness.signature.to_hex(),
        })
    }

    pub fn to_certificate_event(&self, cert: &MultiEraCert) -> Option<EventData> {
        if !cert.as_alonzo().is_some() {
            return None;
        }

        let evt = match cert.as_alonzo().unwrap() {
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

    pub(crate) fn append_rollback_event(&self, point: &Point) -> Result<(), Error> {
        let data = match point {
            Point::Origin => EventData::RollBack {
                block_slot: 0,
                block_hash: "".to_string(),
            },
            Point::Specific(slot, hash) => EventData::RollBack {
                block_slot: *slot,
                block_hash: hex::encode(hash),
            },
        };

        self.append(data)
    }
}
