use aws_sdk_s3::{types::ByteStream, Client};
use std::sync::Arc;

use crate::{
    model::{BlockRecord, EventData},
    pipelining::StageReceiver,
    utils::Utils,
    Error,
};

use super::{ContentType, Naming};

async fn send_s3_object(
    client: Arc<Client>,
    bucket: &str,
    key: &str,
    content: ByteStream,
    content_type: &ContentType,
    record: &BlockRecord,
) -> Result<(), Error> {
    let req = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(content)
        .metadata("era", record.era.to_string())
        .metadata("issuer_vkey", &record.issuer_vkey)
        .metadata("tx_count", record.tx_count.to_string())
        .metadata("slot", record.slot.to_string())
        .metadata("hash", &record.hash)
        .metadata("number", record.number.to_string())
        .metadata("previous_hash", &record.previous_hash)
        .content_type(content_type);

    let res = req.send().await?;

    log::trace!("S3 put response: {:?}", res);

    Ok(())
}

fn define_obj_key(prefix: &str, policy: &Naming, record: &BlockRecord) -> String {
    match policy {
        Naming::Hash => format!("{}{}", prefix, record.hash),
        Naming::SlotHash => format!("{}{}.{}", prefix, record.slot, record.hash),
        Naming::BlockHash => format!("{}{}.{}", prefix, record.number, record.hash),
        Naming::EpochHash => format!(
            "{}{}.{}",
            prefix,
            record.epoch.unwrap_or_default(),
            record.hash
        ),
        Naming::EpochSlotHash => format!(
            "{}{}.{}.{}",
            prefix,
            record.epoch.unwrap_or_default(),
            record.slot,
            record.hash
        ),
        Naming::EpochBlockHash => {
            format!(
                "{}{}.{}.{}",
                prefix,
                record.epoch.unwrap_or_default(),
                record.number,
                record.hash
            )
        }
    }
}

fn define_content(content_type: &ContentType, record: &BlockRecord) -> ByteStream {
    let hex = match record.cbor_hex.as_ref() {
        Some(x) => x,
        None => {
            log::error!(
                "found block record without CBOR, please enable CBOR in source mapper options"
            );
            panic!()
        }
    };

    match content_type {
        ContentType::Cbor => {
            let cbor = hex::decode(hex).expect("valid hex value");
            ByteStream::from(cbor)
        }
        ContentType::CborHex => ByteStream::from(hex.as_bytes().to_vec()),
    }
}

pub fn writer_loop(
    input: StageReceiver,
    client: Client,
    bucket: &str,
    prefix: &str,
    naming: Naming,
    content_type: ContentType,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    for event in input.iter() {
        if let EventData::Block(record) = &event.data {
            let key = define_obj_key(prefix, &naming, record);
            let content = define_content(&content_type, record);

            let client = client.clone();

            let result = rt.block_on(send_s3_object(
                client,
                bucket,
                &key,
                content,
                &content_type,
                record,
            ));

            match result {
                Ok(_) => {
                    // notify the pipeline where we are
                    utils.track_sink_progress(&event);
                }
                Err(err) => {
                    log::error!("unrecoverable error sending block to S3: {:?}", err);
                    return Err(err);
                }
            }
        }
    }

    Ok(())
}
