#![allow(unused_variables)]
use std::sync::Arc;
use serde_json::{json,Value};
use crate::{pipelining::StageReceiver, utils::Utils, Error, model::Event, model::EventData};

pub fn producer_loop(
    input       : StageReceiver,
    utils       : Arc<Utils>,
    conn        : &mut redis::Connection,
) -> Result<(), Error> {
    for event in input.iter() {
        utils.track_sink_progress(&event);
        let value = json!(&event);
        let key = make_key(&event)?;
        log::info!("Key: {:?}, Event: {:?}",key,event);
        if let Some(k) = key{
            //Needs minimum redis 6.9 (release canidate 7.0)
            let _ : () = redis::cmd("XADD").arg(event.data.to_string().to_lowercase()).arg("*").arg(&[(k,Value::to_string(&value))]).query(conn)?;
        }
    }
    Ok(())
}

fn make_key(event :  &Event) -> Result<Option<String>,Error> {
    match event.data.clone() {
        EventData::Block(_) => {
            Ok(event.context.block_number.map(|n| n.to_string()))
        }

        EventData::BlockEnd(_) => {
            Ok(None)
        },

        EventData::Transaction(_) => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::TransactionEnd(_) => {
            Ok(None)
        },

        EventData::TxInput(tx_input_record) => {
            Ok(Some(tx_input_record.tx_id +"#"+&tx_input_record.index.to_string()).map(|n| n.to_string()))
        },

        EventData::TxOutput(_) => {
            let mut key = match event.context.tx_hash.clone(){
                Some(hash) => hash,
                None => return Err("no txhash found".into()),
            };
            if let Some(outputindex) = event.context.output_idx {
                key = key + "#" + &outputindex.to_string()
            }

            Ok(Some(key))
        },

        EventData::OutputAsset(output_asset_record) => {
            Ok(Some(output_asset_record.policy.clone()+"."+&output_asset_record.asset).map(|n| n.to_string()))
        },

        EventData::Metadata(_) => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::CIP25Asset(_) => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::Mint(_) => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::Collateral {
            tx_id,
            index,
        } => {
            Ok(Some(tx_id +"#"+&index.to_string()).map(|n| n.to_string()))
        },

        EventData::NativeScript {
            ..
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::PlutusScript {
            ..
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::StakeRegistration {
            ..
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::StakeDeregistration {
            ..
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::StakeDelegation {
            credential,
            pool_hash,
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string() + "|" + &pool_hash))
        },

        EventData::PoolRegistration {
            ..
        }=> {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::PoolRetirement {
            pool,
            epoch,
        } => {
            Ok(Some(pool))
        },

        EventData::GenesisKeyDelegation => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::MoveInstantaneousRewardsCert {
            ..
        } => {
            Ok(event.context.tx_hash.clone().map(|n| n.to_string()))
        },

        EventData::RollBack {
            ..
        } => {
            Ok(event.context.block_number.map(|n| n.to_string()))
        },
    }
}