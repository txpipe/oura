use std::sync::Arc;

use cloud_pubsub::{error::Error, Client, Topic};
use serde_json::json;

use crate::{
    model::Event,
    pipelining::StageReceiver,
    sinks::ErrorPolicy,
    utils::{retry, Utils},
};

async fn send_pubsub_msg(client: &Topic, event: &Event) -> Result<(), Error> {
    let body = json!(event).to_string();

    client.publish(body).await?;

    Ok(())
}

pub fn writer_loop(
    input: StageReceiver,
    credentials: String,
    topic_name: String,
    error_policy: &ErrorPolicy,
    retry_policy: &retry::Policy,
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

        let result = retry::retry_operation(
            || rt.block_on(send_pubsub_msg(&topic, &event)),
            retry_policy,
        );

        match result {
            Ok(()) => (),
            Err(err) => match error_policy {
                ErrorPolicy::Exit => return Err(Box::new(err)),
                ErrorPolicy::Continue => {
                    log::warn!("failed to publish to pubsub: {:?}", err);
                }
            },
        }
    }

    Ok(())
}
