#![allow(unused_variables)]
use std::sync::Arc;
use serde::Serialize;
use serde_json::{json};
use crate::{pipelining::StageReceiver, utils::Utils, Error, model::Event, model::EventData};

#[derive(Serialize)]
pub struct RedisRecord {
    pub event: Event, 
    pub variant: String,
    pub key: Option<String>,
}

impl From<Event> for RedisRecord {
    fn from(event: Event) -> Self {
        let variant = event.data.to_string().to_lowercase();
        let key = make_key(&event);
        RedisRecord {
            event,
            variant,
            key,
        }
    }
    
}

pub fn producer_loop(
    input       : StageReceiver,
    utils       : Arc<Utils>,
    conn        : &mut redis::Connection,
) -> Result<(), Error> {
    for event in input.iter() {
        utils.track_sink_progress(&event);
        let value = RedisRecord::from(event);
        log::info!("Variant: {:?}, Key: {:?}, Event: {:?}", value.variant, value.key, value.event);
        if let Some(k) = value.key{
            let _ : () = redis::cmd("XADD").arg(value.variant).arg("*").arg(&[(k,json!(value.event).to_string())]).query(conn)?;
        }
    }
    Ok(())
}

fn make_key(event :  &Event) -> Option<String> {
    if event.fingerprint.is_some() {
        return event.fingerprint.clone()
    }
    match event.data.clone() {
        EventData::Block(_) => {
            event.context.block_number.map(|n| n.to_string())
        }

        EventData::BlockEnd(_) => {
            None
        },

        EventData::Transaction(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::TransactionEnd(_) => {
            None
        },

        EventData::TxInput(tx_input_record) => {
            Some(tx_input_record.tx_id +"#"+&tx_input_record.index.to_string()).map(|n| n.to_string())
        },

        EventData::TxOutput(_) => {
            let mut key = match event.context.tx_hash.clone(){
                Some(hash) => hash,
                None => return None,
            };
            if let Some(outputindex) = event.context.output_idx {
                key = key + "#" + &outputindex.to_string()
            }

            Some(key)
        },

        EventData::OutputAsset(output_asset_record) => {
            Some(output_asset_record.policy.clone()+"."+&output_asset_record.asset).map(|n| n.to_string())
        },

        EventData::Metadata(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::CIP25Asset(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::Mint(_) => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::Collateral {
            tx_id,
            index,
        } => {
            Some(tx_id +"#"+&index.to_string()).map(|n| n.to_string())
        },

        EventData::NativeScript {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::PlutusScript {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeRegistration {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeDeregistration {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::StakeDelegation {
            credential,
            pool_hash,
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string() + "|" + &pool_hash)
        },

        EventData::PoolRegistration {
            ..
        }=> {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::PoolRetirement {
            pool,
            epoch,
        } => {
            Some(pool)
        },

        EventData::GenesisKeyDelegation => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::MoveInstantaneousRewardsCert {
            ..
        } => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },

        EventData::RollBack {
            ..
        } => {
            event.context.block_number.map(|n| n.to_string())
        },
    }
}