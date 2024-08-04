use std::{collections::HashMap, sync::Arc};

use google_cloud_gax::conn::Environment;
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

pub type GenericKV = HashMap<String, String>;

async fn send_pubsub_msg(
    publisher: &Publisher,
    event: &Event,
    ordering_key: &str,
    attributes: &GenericKV,
) -> Result<(), crate::Error> {
    let body = json!(event).to_string();
    let msg = PubsubMessage {
        data: body.into(),
        ordering_key: ordering_key.into(),
        attributes: attributes.clone(),
        ..Default::default()
    };

    publisher
        .publish_immediately(vec![msg], None)
        .await
        .map_err(|err| err.message().to_owned())?;

    log::debug!("gcp message sent");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn writer_loop(
    input: StageReceiver,
    topic_name: &str,
    error_policy: &ErrorPolicy,
    retry_policy: &retry::Policy,
    ordering_key: &str,
    attributes: &GenericKV,
    emulator: bool,
    emulator_endpoint: &Option<String>,
    emulator_project_id: &Option<String>,
    utils: Arc<Utils>,
) -> Result<(), crate::Error> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    let publisher: Publisher = rt.block_on(async {
        let client_config = if emulator {
            ClientConfig {
                project_id: Some(emulator_project_id.clone().unwrap_or_default()),
                environment: Environment::Emulator(emulator_endpoint.clone().unwrap_or_default()),
                ..Default::default()
            }
        } else {
            ClientConfig::default()
        };
        let client = Client::new(client_config).await?;
        let topic = client.topic(topic_name);
        Result::<_, crate::Error>::Ok(topic.new_publisher(None))
    })?;

    for event in input.iter() {
        let result = retry::retry_operation(
            || {
                rt.block_on(send_pubsub_msg(
                    &publisher,
                    &event,
                    ordering_key,
                    attributes,
                ))
            },
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
