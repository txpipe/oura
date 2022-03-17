use aws_sdk_sqs::Client;
use serde_json::json;
use std::sync::Arc;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

async fn send_sqs_msg(
    client: Arc<Client>,
    queue_url: &str,
    group_id: &str,
    fifo: bool,
    event: &Event,
) -> Result<(), Error> {
    let body = json!(event).to_string();

    let mut req = client
        .send_message()
        .queue_url(queue_url)
        .message_body(body);

    if fifo {
        req = req.message_group_id(group_id);

        if let Some(id) = &event.fingerprint {
            req = req.message_deduplication_id(id)
        }
    }

    let res = req.send().await?;

    log::trace!("SQS send response: {:?}", res);

    Ok(())
}

pub fn writer_loop(
    input: StageReceiver,
    client: Client,
    queue_url: &str,
    fifo: bool,
    group_id: &str,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    loop {
        let event = input.recv().unwrap();

        // notify the pipeline where we are
        utils.track_sink_progress(&event);

        let client = client.clone();

        let result = rt.block_on(send_sqs_msg(client, queue_url, group_id, fifo, &event));

        if let Err(err) = result {
            log::error!("unrecoverable error sending message to SQS: {:?}", err);
            break Err(err);
        }
    }
}
