use std::sync::Arc;

use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    publisher::Publisher,
};

use serde_json::json;

use crate::{
    model::Event,
    pipelining::StageReceiver,
    sinks::common::web::ErrorPolicy,
    utils::{retry, Utils},
};

async fn send_pubsub_msg(publisher: &Publisher, event: &Event) -> Result<(), crate::Error> {
    let body = json!(event).to_string();
    let msg = PubsubMessage {
        data: body.into(),
        ..Default::default()
    };

    publisher
        .publish_immediately(vec![msg], None, None)
        .await
        .map_err(|err| err.message().to_owned())?;

    log::debug!("gcp message sent");

    Ok(())
}

pub fn writer_loop(
    input: StageReceiver,
    topic_name: &str,
    error_policy: &ErrorPolicy,
    retry_policy: &retry::Policy,
    utils: Arc<Utils>,
) -> Result<(), crate::Error> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    let publisher: Publisher = rt.block_on(async {
        let client = Client::new(ClientConfig::default()).await?;
        let topic = client.topic(topic_name);
        Result::<_, crate::Error>::Ok(topic.new_publisher(None))
    })?;

    for event in input.iter() {
        let result = retry::retry_operation(
            || rt.block_on(send_pubsub_msg(&publisher, &event)),
            retry_policy,
        );

        match result {
            Ok(_) => {
                // notify the pipeline where we are
                utils.track_sink_progress(&event);
            }
            Err(err) => match error_policy {
                ErrorPolicy::Exit => return Err(err),
                ErrorPolicy::Continue => {
                    log::warn!("failed to publish to pubsub: {:?}", err);
                }
            },
        }
    }

    Ok(())
}
