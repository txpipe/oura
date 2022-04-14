use std::sync::Arc;

use cloud_pubsub::{error::Error, topic::PublishMessageResponse, Client, Topic};
use serde_json::json;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils};

async fn send_pubsub_msg(client: &Topic, event: &Event) -> Result<PublishMessageResponse, Error> {
    let body = json!(event).to_string();

    client.publish(body).await
}

pub fn writer_loop(
    input: StageReceiver,
    credentials: String,
    topic_name: String,
    utils: Arc<Utils>,
) -> Result<(), crate::Error> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    let publisher = rt.block_on(Client::new(credentials))?;
    let topic = publisher.topic(topic_name);

    for event in input.iter() {
        // notify the pipeline where we are
        utils.track_sink_progress(&event);

        let result = rt.block_on(send_pubsub_msg(&topic, &event));

        if let Err(err) = result {
            log::error!("unrecoverable error sending message to PubSub: {:?}", err);
            return Err(Box::new(err));
        }
    }

    Ok(())
}
