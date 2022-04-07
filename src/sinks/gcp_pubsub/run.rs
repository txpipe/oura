use std::{sync::Arc, time::SystemTime};

use google_cloud_rust_raw::pubsub::v1::{
    pubsub::{PublishRequest, PublishResponse, PubsubMessage, Topic},
    pubsub_grpc::PublisherClient,
};
use protobuf::RepeatedField;
use serde_json::json;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

fn send_pubsub_msg(
    client: &PublisherClient,
    topic: &Topic,
    event: &Event,
) -> ::grpcio::Result<PublishResponse> {
    let body = json!(event).to_string();

    let timestamp_in_seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut timestamp = protobuf::well_known_types::Timestamp::new();
    timestamp.set_seconds(timestamp_in_seconds as i64);

    let mut pubsub_msg = PubsubMessage::new();
    pubsub_msg.set_data(body.into_bytes());
    pubsub_msg.set_publish_time(timestamp);

    let mut request = PublishRequest::new();
    request.set_topic(topic.get_name().to_string());
    request.set_messages(RepeatedField::from_vec(vec![pubsub_msg]));

    client.publish(&request)
}

pub fn writer_loop(
    input: StageReceiver,
    publisher: PublisherClient,
    topic: Topic,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    for event in input.iter() {
        // notify the pipeline where we are
        utils.track_sink_progress(&event);

        let result = send_pubsub_msg(&publisher, &topic, &event);

        if let Err(err) = result {
            log::error!("unrecoverable error sending message to PubSub: {:?}", err);
            return Err(Box::new(err));
        }
    }

    Ok(())
}
