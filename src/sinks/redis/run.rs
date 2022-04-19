#![allow(unused_variables)]
use std::sync::Arc;
use crate::{pipelining::StageReceiver, utils::Utils, Error, model::Event, model::EventData};

pub fn producer_loop(
    input       : StageReceiver,
    utils       : Arc<Utils>,
    conn        : &mut redis::Connection,
) -> Result<(), Error> {
    
    for event in input.iter() {
        utils.track_sink_progress(&event);

        let value = serde_json::to_string(&event)?;
        let (stream, key) = data(&event)?;
        
        log::info!("Key: {:?}, Event: {:?}",key, event);
        if let Some(k) = key {
            //Needs minimum redis 6.9 (release canidate 7.0) 
            let _ : () = redis::cmd("XADD").arg(stream).arg("*").arg(&[(k,value)]).query(conn)?;
        }
    }

    Ok(())
}

fn data(event :  &Event) -> Result<(String,Option<String>),Error> {
    match event.data.clone() {
        EventData::Block(_) => {
            Ok(("block".to_string(), event.context.block_number.map(|n| n.to_string())))
        }
        EventData::BlockEnd(_) => { 
            Ok(("blockend".to_string(),None))
        },

        EventData::Transaction(_) => {
            Ok(("transaction".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::TransactionEnd(_) => {
            Ok(("txend".to_string(),None))
        },

        EventData::TxInput(tx_input_record) => {
            Ok(("txin".to_string(),Some(tx_input_record.tx_id +"#"+&tx_input_record.index.to_string()).map(|n| n.to_string())))
        },

        EventData::TxOutput(_) => {
            let mut key = match event.context.tx_hash.clone(){
                Some(hash) => hash,
                None => return Err("no txhash found".into()),
            };
            if let Some(outputindex) = event.context.output_idx {
                key = key + "#" + &outputindex.to_string()
            }

            Ok(("txout".to_string(),Some(key)))
        },

        EventData::OutputAsset(output_asset_record) => {
            Ok(("outputseets".to_string(),Some(output_asset_record.policy.clone()+"."+&output_asset_record.asset).map(|n| n.to_string())))
        },

        EventData::Metadata(_) => {
            Ok(("metadata".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::CIP25Asset(_) => {
            Ok(("cip25mint".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::Mint(_) => {
            Ok(("mint".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::Collateral { 
            tx_id, 
            index,
        } => {
            Ok(("collateral".to_string(),Some(tx_id +"#"+&index.to_string()).map(|n| n.to_string())))
        },

        EventData::NativeScript {} => {
            Ok(("nativ_script_tx".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::PlutusScript {
            ..
        } => {
            Ok(("smart_contract_tx".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::StakeRegistration {
            ..
        } => {
            Ok(("stakeregistration".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::StakeDeregistration {
            ..
        } => {
            Ok(("stakederegistration".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::StakeDelegation {
            credential, 
            pool_hash, 
        } => {
            Ok(("stakedelegation".to_string(),event.context.tx_hash.clone().map(|n| n.to_string() + "|" + &pool_hash)))
        },
        
        EventData::PoolRegistration {
            ..
        }=> {
            Ok(("poolregistration".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::PoolRetirement { 
            pool, 
            epoch,
        } => {
            Ok(("poolretirement".to_string(),Some(pool)))
        },

        EventData::GenesisKeyDelegation => {
            Ok(("genesiskeydelegation".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::MoveInstantaneousRewardsCert {
            ..
        } => {
            Ok(("moveinstrewardcert".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::RollBack {
           .. 
        } => {
            Ok(("rollback".to_string(),event.context.block_number.map(|n| n.to_string())))
        },
    }
}