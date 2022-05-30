use std::{sync::Arc, time::Duration};

use async_recursion::async_recursion;
use cloud_pubsub::{error::Error, Client, Topic};
use serde_json::json;

use crate::{model::Event, pipelining::StageReceiver, sinks::ErrorPolicy, utils::Utils};

#[async_recursion]
async fn send_pubsub_msg(
    client: &Topic,
    event: &Event,
    policy: &ErrorPolicy,
    retry_quota: usize,
    backoff_delay: Duration,
) -> Result<(), Error> {
    let body = json!(event).to_string();

    let result = client.publish(body).await;

    match (result, policy, retry_quota) {
        (Ok(_), _, _) => {
            log::info!("successful pubsub publish");
            Ok(())
        }
        (Err(x), ErrorPolicy::Exit, 0) => Err(x),
        (Err(x), ErrorPolicy::Continue, 0) => {
            log::warn!("failed to publish to pubsub: {:?}", x);
            Ok(())
        }
        (Err(x), _, quota) => {
            log::warn!("failed attempt to execute pubsub publish: {:?}", x);
            std::thread::sleep(backoff_delay);
            send_pubsub_msg(client, event, policy, quota - 1, backoff_delay).await
        }
    }
}

pub fn writer_loop(
    input: StageReceiver,
    credentials: String,
    topic_name: String,
    error_policy: &ErrorPolicy,
    max_retries: usize,
    backoff_delay: Duration,
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

        rt.block_on(send_pubsub_msg(
            &topic,
            &event,
            error_policy,
            max_retries,
            backoff_delay,
        ))?;
    }

    Ok(())
}
