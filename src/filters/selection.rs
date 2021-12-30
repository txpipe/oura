//! A filter that can select which events to block and which to let pass

use std::{sync::mpsc::Receiver, thread};

use serde_derive::Deserialize;

use crate::framework::{Event, EventData, FilterConfig, PartialBootstrapResult};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "predicate", content = "argument", rename_all = "snake_case")]
pub enum Predicate {
    VariantIn(Vec<String>),
    VariantNotIn(Vec<String>),
    PolicyEquals(String),
    AssetEquals(String),
    MetadataKeyEquals(String),
    MetadataSubkeyEquals(String),
    Not(Box<Predicate>),
    AnyOf(Vec<Predicate>),
    AllOf(Vec<Predicate>),
}

#[inline]
fn relaxed_str_matches(a: &str, b: &str) -> bool {
    a.to_lowercase().eq(&b.to_lowercase())
}

#[inline]
fn variant_in_matches(event: &Event, variants: &Vec<String>) -> bool {
    variants
        .iter()
        .any(|v| relaxed_str_matches(&event.data.to_string(), v))
}

#[inline]
fn policy_matches(event: &Event, policy: &str) -> bool {
    match &event.data {
        EventData::OutputAsset { policy: x, .. } => relaxed_str_matches(x, &policy),
        EventData::Mint { policy: x, .. } => relaxed_str_matches(x, &policy),
        _ => false,
    }
}

#[inline]
fn asset_matches(event: &Event, asset: &str) -> bool {
    match &event.data {
        EventData::OutputAsset { asset: x, .. } => relaxed_str_matches(x, &asset),
        EventData::Mint { asset: x, .. } => relaxed_str_matches(x, &asset),
        _ => false,
    }
}

#[inline]
fn metadata_key_matches(event: &Event, key: &str) -> bool {
    match &event.data {
        EventData::Metadata { key: x, .. } => relaxed_str_matches(x, &key),
        _ => false,
    }
}

#[inline]
fn metadata_subkey_matches(event: &Event, subkey: &str) -> bool {
    match &event.data {
        EventData::Metadata { subkey: Some(x), .. } => relaxed_str_matches(x, &subkey),
        _ => false,
    }
}

impl Predicate {
    fn event_matches(&self, event: &Event) -> bool {
        match self {
            Predicate::VariantIn(x) => variant_in_matches(event, x),
            Predicate::VariantNotIn(x) => !variant_in_matches(event, x),
            Predicate::PolicyEquals(x) => policy_matches(event, x),
            Predicate::AssetEquals(x) => asset_matches(event, x),
            Predicate::MetadataKeyEquals(x) => metadata_key_matches(event, x),
            Predicate::MetadataSubkeyEquals(x) => metadata_subkey_matches(event, x),
            Predicate::Not(x) => !x.event_matches(event),
            Predicate::AnyOf(x) => x.iter().any(|c| c.event_matches(event)),
            Predicate::AllOf(x) => x.iter().all(|c| c.event_matches(event)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    check: Predicate,
}

impl FilterConfig for Config {
    fn bootstrap(&self, input: Receiver<Event>) -> PartialBootstrapResult {
        let (output_tx, output_rx) = std::sync::mpsc::channel();

        let check = self.check.clone();

        let handle = thread::spawn(move || loop {
            let event = input.recv().expect("error receiving message");
            if check.event_matches(&event) {
                output_tx.send(event).expect("error sending filter message");
            }
        });

        Ok((handle, output_rx))
    }
}
