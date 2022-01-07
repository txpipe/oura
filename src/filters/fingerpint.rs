//! A filter that computes a (probably) unique ID for each event

use std::{
    io::{Cursor, Write},
    thread,
};

use log::{debug, warn};
use serde_derive::Deserialize;

use crate::framework::{
    new_inter_stage_channel, Error, Event, EventData, FilterConfig, MetadataRecord, MintRecord,
    OutputAssetRecord, PartialBootstrapResult, StageReceiver,
};

struct FingerprintBuilder {
    seed: u32,
    prefix: Option<&'static str>,
    slot: Option<u64>,
    hasheable: Vec<u8>,
}

impl FingerprintBuilder {
    fn new(seed: u32) -> Self {
        FingerprintBuilder {
            seed,
            prefix: None,
            slot: None,
            hasheable: Vec::with_capacity(50),
        }
    }

    fn with_slot(mut self, slot: &Option<u64>) -> Self {
        self.slot = *slot;
        self
    }

    fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    fn append_slice<T>(mut self, value: T) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        self.hasheable.write_all(value.as_ref())?;
        Ok(self)
    }

    fn append_optional<T>(mut self, value: &Option<T>) -> Result<Self, Error>
    where
        T: AsRef<[u8]>,
    {
        match value {
            None => Err("fingerprint component not available".into()),
            Some(x) => {
                let slice = x.as_ref();
                self.hasheable.write_all(slice)?;
                Ok(self)
            }
        }
    }

    fn append_optional_to_string<T>(self, value: &Option<T>) -> Result<Self, Error>
    where
        T: ToString,
    {
        let mapped = value.as_ref().map(|x| x.to_string());
        self.append_optional(&mapped)
    }

    fn append_to_string<T>(self, value: &T) -> Result<Self, Error>
    where
        T: ToString,
    {
        let str = value.to_string();
        self.append_slice(str)
    }

    fn build(self) -> Result<String, Error> {
        let slot = self.slot.ok_or("missing slot value")?;
        let prefix = self.prefix.ok_or("missing prefix value")?;
        let hash = murmur3::murmur3_x64_128(&mut Cursor::new(self.hasheable), self.seed)?;
        Ok(format!("{}.{}.{}", slot, prefix, hash))
    }
}

#[inline]
fn build_fingerprint(event: &Event, seed: u32) -> Result<String, Error> {
    let mut b = FingerprintBuilder::new(seed);

    b = match &event.data {
        EventData::Block { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("blck")
            .append_optional(&event.context.block_hash)?,
        EventData::Transaction { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("tx")
            .append_optional(&event.context.tx_hash)?,
        EventData::TxInput { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("stxi")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.input_idx)?,
        EventData::TxOutput { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("utxo")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.output_idx)?,
        EventData::OutputAsset(OutputAssetRecord { policy, asset, .. }) => b
            .with_slot(&event.context.slot)
            .with_prefix("asst")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.output_idx)?
            .append_slice(policy)?
            .append_slice(asset)?,
        EventData::Metadata(MetadataRecord { label, .. }) => b
            .with_slot(&event.context.slot)
            .with_prefix("meta")
            .append_optional(&event.context.tx_hash)?
            .append_slice(label)?,
        EventData::Mint(MintRecord { policy, asset, .. }) => b
            .with_slot(&event.context.slot)
            .with_prefix("mint")
            .append_optional(&event.context.tx_hash)?
            .append_slice(policy)?
            .append_slice(asset)?,
        EventData::Collateral { tx_id, index } => b
            .with_slot(&event.context.slot)
            .with_prefix("coll")
            .append_slice(tx_id)?
            .append_to_string(index)?,
        EventData::NativeScript => b
            .with_slot(&event.context.slot)
            .with_prefix("scpt")
            .append_optional(&event.context.tx_hash)?,
        EventData::PlutusScript { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("plut")
            .append_optional(&event.context.tx_hash)?,
        EventData::StakeRegistration { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("skre")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::StakeDeregistration { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("skde")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::StakeDelegation { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("dele")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::PoolRegistration { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("pool")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::PoolRetirement { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("reti")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::GenesisKeyDelegation => b
            .with_slot(&event.context.slot)
            .with_prefix("gene")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::MoveInstantaneousRewardsCert { .. } => b
            .with_slot(&event.context.slot)
            .with_prefix("move")
            .append_optional(&event.context.tx_hash)?
            .append_optional_to_string(&event.context.certificate_idx)?,
        EventData::RollBack {
            block_slot,
            block_hash,
        } => b
            .with_slot(&Some(*block_slot))
            .with_prefix("back")
            .append_slice(block_hash)?,
    };

    b.build()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub seed: Option<u32>,
}

impl FilterConfig for Config {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let seed = self.seed.unwrap_or(0);

        let handle = thread::spawn(move || loop {
            let mut msg = input.recv().expect("error receiving message");

            let fingerprint = build_fingerprint(&msg, seed);

            match fingerprint {
                Ok(value) => {
                    debug!("computed fingerprint {}", value);
                    msg.fingerprint = Some(value);
                    output_tx.send(msg).expect("error sending filter message");
                }
                Err(err) => {
                    warn!("failed to compute fingerprint: {}, event: {:?}", err, msg);
                }
            }
        });

        Ok((handle, output_rx))
    }
}
