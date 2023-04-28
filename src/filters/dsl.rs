//! A filter that can select which events to block and which to let pass

use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::{framework::legacy_v1::*, framework::*};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "predicate", content = "argument", rename_all = "snake_case")]
pub enum Predicate {
    VariantIn(Vec<String>),
    VariantNotIn(Vec<String>),
    PolicyEquals(String),
    AssetEquals(String),
    AddressEquals(String),
    MetadataLabelEquals(String),
    MetadataAnySubLabelEquals(String),
    VKeyWitnessesIncludes(String),
    NativeScriptsIncludes(String),
    PlutusScriptsIncludes(String),
    Not(Box<Predicate>),
    AnyOf(Vec<Predicate>),
    AllOf(Vec<Predicate>),
}

#[inline]
fn relaxed_str_matches(a: &str, b: &str) -> bool {
    a.to_lowercase().eq(&b.to_lowercase())
}

#[inline]
fn variant_in_matches(event: &Event, variants: &[String]) -> bool {
    variants
        .iter()
        .any(|v| relaxed_str_matches(&event.data.to_string(), v))
}

#[inline]
fn output_policy_matches(event: &Event, policy: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            outputs: Some(outputs),
            ..
        }) => outputs
            .iter()
            .flat_map(|x| &x.assets)
            .flatten()
            .any(|x| relaxed_str_matches(&x.policy, policy)),
        EventData::OutputAsset(OutputAssetRecord { policy: x, .. }) => {
            relaxed_str_matches(x, policy)
        }
        _ => false,
    }
}

#[inline]
fn mint_policy_matches(event: &Event, policy: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            mint: Some(mint), ..
        }) => mint.iter().any(|x| relaxed_str_matches(&x.policy, policy)),
        EventData::OutputAsset(OutputAssetRecord { policy: x, .. }) => {
            relaxed_str_matches(x, policy)
        }
        EventData::Mint(MintRecord { policy: x, .. }) => relaxed_str_matches(x, policy),
        _ => false,
    }
}

#[inline]
fn cip25_policy_matches(event: &Event, policy: &str) -> bool {
    match &event.data {
        EventData::CIP25Asset(CIP25AssetRecord { policy: x, .. }) => relaxed_str_matches(x, policy),
        _ => false,
    }
}

#[inline]
fn address_matches(event: &Event, address: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            outputs: Some(o), ..
        }) => o.iter().any(|x| relaxed_str_matches(&x.address, address)),
        EventData::TxOutput(TxOutputRecord { address: x, .. }) => relaxed_str_matches(x, address),
        _ => false,
    }
}

#[inline]
fn output_asset_matches(event: &Event, asset: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            outputs: Some(outputs),
            ..
        }) => outputs
            .iter()
            .flat_map(|x| &x.assets)
            .flatten()
            .any(|x| relaxed_str_matches(&x.asset, asset)),
        EventData::OutputAsset(OutputAssetRecord { asset: x, .. }) => relaxed_str_matches(x, asset),
        _ => false,
    }
}

#[inline]
fn mint_asset_matches(event: &Event, asset: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            mint: Some(mint), ..
        }) => mint.iter().any(|x| relaxed_str_matches(&x.asset, asset)),
        EventData::Mint(MintRecord { asset: x, .. }) => relaxed_str_matches(x, asset),
        _ => false,
    }
}

#[inline]
fn cip25_asset_matches(event: &Event, asset: &str) -> bool {
    match &event.data {
        EventData::CIP25Asset(CIP25AssetRecord { asset: x, .. }) => relaxed_str_matches(x, asset),
        _ => false,
    }
}

#[inline]
fn metadata_label_matches(event: &Event, label: &str) -> bool {
    match &event.data {
        EventData::Transaction(TransactionRecord {
            metadata: Some(x), ..
        }) => x.iter().any(|r| relaxed_str_matches(&r.label, label)),
        EventData::Metadata(MetadataRecord { label: x, .. }) => relaxed_str_matches(x, label),
        _ => false,
    }
}

#[inline]
fn metadata_any_sub_label_matches(event: &Event, sub_label: &str) -> bool {
    match &event.data {
        EventData::Metadata(record) => match &record.content {
            MetadatumRendition::MapJson(JsonValue::Object(obj)) => {
                obj.keys().any(|x| relaxed_str_matches(x, sub_label))
            }
            _ => false,
        },
        _ => false,
    }
}

#[inline]
fn vkey_witnesses_matches(event: &Event, witness: &str) -> bool {
    match &event.data {
        EventData::VKeyWitness(x) => x.vkey_hex == witness,
        EventData::Transaction(x) => x
            .vkey_witnesses
            .as_ref()
            .map(|vs| vs.iter().any(|v| v.vkey_hex == witness))
            .unwrap_or(false),
        _ => false,
    }
}

#[inline]
fn native_scripts_matches(event: &Event, policy_id: &str) -> bool {
    match &event.data {
        EventData::NativeWitness(x) => x.policy_id == policy_id,
        EventData::Transaction(x) => x
            .native_witnesses
            .as_ref()
            .map(|vs| vs.iter().any(|v| v.policy_id == policy_id))
            .unwrap_or(false),
        _ => false,
    }
}

#[inline]
fn plutus_scripts_matches(event: &Event, script_hash: &str) -> bool {
    match &event.data {
        EventData::PlutusWitness(x) => x.script_hash == script_hash,
        EventData::Transaction(x) => x
            .plutus_witnesses
            .as_ref()
            .map(|vs| vs.iter().any(|v| v.script_hash == script_hash))
            .unwrap_or(false),
        _ => false,
    }
}

impl Predicate {
    #![allow(deprecated)]
    fn event_matches(&self, event: &Event) -> bool {
        match self {
            Predicate::VariantIn(x) => variant_in_matches(event, x),
            Predicate::VariantNotIn(x) => !variant_in_matches(event, x),
            Predicate::PolicyEquals(x) => {
                output_policy_matches(event, x)
                    || mint_policy_matches(event, x)
                    || cip25_policy_matches(event, x)
            }
            Predicate::AddressEquals(x) => address_matches(event, x),
            Predicate::AssetEquals(x) => {
                output_asset_matches(event, x)
                    || mint_asset_matches(event, x)
                    || cip25_asset_matches(event, x)
            }
            Predicate::MetadataLabelEquals(x) => metadata_label_matches(event, x),
            Predicate::MetadataAnySubLabelEquals(x) => metadata_any_sub_label_matches(event, x),
            Predicate::VKeyWitnessesIncludes(x) => vkey_witnesses_matches(event, x),
            Predicate::NativeScriptsIncludes(x) => native_scripts_matches(event, x),
            Predicate::PlutusScriptsIncludes(x) => plutus_scripts_matches(event, x),
            Predicate::Not(x) => !x.event_matches(event),
            Predicate::AnyOf(x) => x.iter().any(|c| c.event_matches(event)),
            Predicate::AllOf(x) => x.iter().all(|c| c.event_matches(event)),
        }
    }
}

pub struct Stage {
    ops_count: gasket::metrics::Counter,
    predicate: Predicate,
    input: FilterInputPort,
    output: FilterOutputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "filter"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
    }
}

gasket::stateless_flatmapper!(Worker, |stage: Stage, unit: ChainEvent| => {
    let out = match unit {
        ChainEvent::Apply(_, Record::OuraV1Event(x)) => {
            if stage.predicate.event_matches(x) {
                Some(unit.clone())
            } else {
                None
            }
        }
        _ => todo!(),
    };

    stage.ops_count.inc(1);
    out
});

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub predicate: Predicate,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            predicate: self.predicate,
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
