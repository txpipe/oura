use std::sync::Arc;

//use redis::*;
//use log::debug;

use crate::{pipelining::StageReceiver, utils::Utils, Error, model::Event, model::EventData};
use super::PartitionStrategy;

pub fn producer_loop(
    input       : StageReceiver,
    utils       : Arc<Utils>,
    conn        : &mut redis::Connection,
) -> Result<(), Error> {
    
    for event in input.iter() {
        utils.track_sink_progress(&event);

        let value = serde_json::to_string(&event)?;
        let (stream, key) = data(&event)?;
        
        info!("Key: {:?}, Event: {:?}",key, event);
        if let Some(k) = key {
            let _ : () = redis::cmd("XADD").arg(stream).arg("*").arg(&[(k,value)]).query(conn)?;
        }
    }

    Ok(())
}

fn data(event :  &Event) -> Result<(String,Option<String>),Error> {
    match event.data.clone() {
        EventData::Block(BlockRecord) => {
            Ok(("block".to_string(), event.context.block_number.map(|n| n.to_string())))
        }
        EventData::BlockEnd(BlockRecord) => { 
            Ok(("".to_string(),None))
        },

        EventData::Transaction(TransactionRecord) => {
            Ok(("transaction".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::TransactionEnd(TransactionRecord) => {
            Ok(("".to_string(),None))
        },

        EventData::TxInput(TxInputRecord) => {
            Ok(("".to_string(),None))
        },

        EventData::TxOutput(TxOutputRecord) => {
            Ok(("".to_string(),None))
        },

        EventData::OutputAsset(OutputAssetRecord) => {
            Ok(("".to_string(),None))
        },

        EventData::Metadata(MetadataRecord) => {
            Ok(("".to_string(),None))
        },

        EventData::CIP25Asset(CIP25AssetRecord) => {
            Ok(("cip25mint".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::Mint(MintRecord) => {
            Ok(("mint".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },

        EventData::Collateral { 
            tx_id, //: String, 
            index, //: u64,
        } => {
            Ok(("".to_string(),None))
        },

        EventData::NativeScript {} => {
            Ok(("nativ_script_tx".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::PlutusScript {
            data, //: String,
        } => {
            Ok(("smart_contract_tx".to_string(),event.context.tx_hash.clone().map(|n| n.to_string())))
        },
        
        EventData::StakeRegistration {
            credential, //: StakeCredential,
        } => {
            Ok(("".to_string(),None))
        },
        
        EventData::StakeDeregistration {
            credential, //: StakeCredential, 
        } => {
            Ok(("".to_string(),None))
        },
        
        EventData::StakeDelegation {
            credential, //: StakeCredential, 
            pool_hash, //: String, 
        } => {
            Ok(("".to_string(),None))
        },
        
        EventData::PoolRegistration {
            operator, //: String,
            vrf_keyhash, //: String,
            pledge, //: u64,
            cost, //: u64,
            margin, //: f64,
            reward_account, //: String,
            pool_owners, //: Vec::<String>,
            relays, //: Vec::<String>,
            pool_metadata, //: Option<String>,
        }=> {
            Ok(("".to_string(),None))
        },
        
        EventData::PoolRetirement { 
            pool, //: String, 
            epoch, //: u64, 
        } => {
            Ok(("".to_string(),None))
        },

        EventData::GenesisKeyDelegation => {
            Ok(("".to_string(),None))
        },

        EventData::MoveInstantaneousRewardsCert {
            from_reserves, //: bool,
            from_treasury, //: bool,
            to_stake_credentials, //: Option::<Vec::<(StakeCredential, i64)>>,
            to_other_pot, //: Option<u64>,
        } => {
            Ok(("".to_string(),None))
        },

        EventData::RollBack {
            block_slot, //: u64, 
            block_hash, //: String,
        } => {
            Ok(("rollback".to_string(),event.context.block_number.map(|n| n.to_string())))
        },
    }
}

struct Transaction {

}