use pallas::ledger::alonzo::{
    crypto, AuxiliaryData, Block, Metadata, TransactionBody, TransactionBodyComponent,
};

use crate::framework::{Error, EventContext};

use super::framework::EventWriter;

fn crawl_metadata_events(metadata: &Metadata, writer: &EventWriter) -> Result<(), Error> {}

fn crawl_auxdata_events(aux_data: &AuxiliaryData, writer: &EventWriter) -> Result<(), Error> {
    match aux_data {
        AuxiliaryData::Alonzo(data) => {
            if let Some(metadata) = &data.metadata {
                crawl_metadata_events(metadata, writer)?;
            }

            for _native in data.native_scripts.iter() {
                // run native-script-level mappers
            }

            if let Some(plutus) = &data.plutus_scripts {
                for script in plutus.iter() {
                    // run plusut-script-level mappers
                }
            }
        }
        AuxiliaryData::Shelley(data) => {
            crawl_metadata_events(data, writer)?;
        }
        AuxiliaryData::ShelleyMa {
            transaction_metadata,
            ..
        } => {
            crawl_metadata_events(transaction_metadata, writer)?;;

            // TODO: process auxiliary scripts
        }
    }

    Ok(())
}

fn crawl_transaction_events(
    tx: &TransactionBody,
    aux_data: Option<&AuxiliaryData>,
    writer: &EventWriter,
) -> Result<(), Error> {
    // run tx-level mappers

    for component in tx.iter() {
        match component {
            TransactionBodyComponent::Inputs(x) => {
                for input in x.iter() {
                    // run input-level mappers
                }
            }
            TransactionBodyComponent::Outputs(x) => {
                for output in x.iter() {
                    // run output-level mappers
                }
            }
            TransactionBodyComponent::Mint(x) => {
                for mint in x.iter() {
                    // run mint-level mappers
                }
            }
            _ => (),
        };
    }

    if let Some(aux_data) = aux_data {
        crawl_auxdata_events(aux_data, writer)?;
    }

    Ok(())
}

fn crawl_block_events(block: &Block, writer: &EventWriter) -> Result<(), Error> {
    let hash = crypto::hash_block_header(&block.header)?;

    let writer = writer.child_writer(EventContext {
        block_hash: Some(hex::encode(hash)),
        block_number: Some(block.header.header_body.block_number),
        slot: Some(block.header.header_body.slot),
        timestamp: writer.compute_timestamp(block.header.header_body.slot),
        ..EventContext::default()
    });

    // run block-level mappers

    for (idx, tx) in block.transaction_bodies.iter().enumerate() {
        let tx_hash = match crypto::hash_transaction(tx) {
            Ok(h) => Some(hex::encode(h)),
            Err(err) => {
                log::warn!("error hashing transaction: {:?}", err);
                None
            }
        };

        let writer = &writer.child_writer(EventContext {
            tx_idx: Some(idx),
            tx_hash: tx_hash.clone(),
            ..EventContext::default()
        });

        let aux_data = block
            .auxiliary_data_set
            .iter()
            .find(|(k, _)| *k == (idx as u32))
            .map(|(_, v)| v);

        crawl_transaction_events(tx, aux_data, writer)?;
    }

    Ok(())
}
