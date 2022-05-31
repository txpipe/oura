#![allow(unused_variables)]
use std::sync::Arc;
use serde::Serialize;
use serde_json::{json};
use crate::{pipelining::StageReceiver, utils::Utils, Error, model::Event, model::EventData};
use super::{StreamConfig};

#[derive(Serialize)]
pub struct RedisRecord {
    pub event: Event, 
    pub stream: String,
    pub key: String,
}

impl From<Event> for RedisRecord {
    fn from(event: Event) -> Self {
        let stream = event.data.to_string().to_lowercase();
        let key = make_key(&event).unwrap_or("undefined".to_string());
        RedisRecord {
            event,
            stream,
            key,
        }
    }
}

pub fn producer_loop(
    input           : StageReceiver,
    utils           : Arc<Utils>,
    conn            : &mut redis::Connection,
    stream_config   : StreamConfig,
    redis_stream    : String,
) -> Result<(), Error> {
    for event in input.iter() {
        utils.track_sink_progress(&event);
        let mut value = RedisRecord::from(event);
        match stream_config {
            StreamConfig::SingleStream => {
                value.stream = redis_stream.clone();
                value.key = value.event.data.clone().to_string();
            },
            _ => {}
        }
        log::debug!("Variant: {:?}, Key: {:?}, Event: {:?}", value.stream, value.key, value.event);
        let _ : () = redis::cmd("XADD").arg(value.stream).arg("*").arg(&[(value.key,json!(value.event).to_string())]).query(conn)?;
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
        },
        EventData::BlockEnd(_) => {
            event.context.block_number.map(|n| n.to_string())
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
        EventData::Collateral {
            tx_id,
            index,
        } => {
            Some(tx_id +"#"+&index.to_string()).map(|n| n.to_string())
        },
        EventData::StakeDelegation {
            credential,
            pool_hash,
        } => {
            Some(pool_hash)
        },
        EventData::RollBack {
            block_slot,
            block_hash,
        } => {
            Some(block_hash)
        },
        _ => {
            event.context.tx_hash.clone().map(|n| n.to_string())
        },
    }
}