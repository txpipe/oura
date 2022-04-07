use std::{collections::HashMap, sync::Arc};

use google_cloud_rust_raw::pubsub::v1::{
    pubsub::{GetTopicRequest, Topic},
    pubsub_grpc::PublisherClient,
};
use grpcio::{Channel, ChannelBuilder, ChannelCredentials, EnvBuilder};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::writer_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub project_id: String,
    pub topic_name: String,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let project_id = self.inner.project_id.to_owned();
        let topic_name = self.inner.topic_name.to_owned();

        let channel = connect("pubsub.googleapis.com");
        let publisher = PublisherClient::new(channel);

        // TODO: do we want to do this in the spawned thread instead?
        let topic_full_name = format!("projects/{project_id}/topics/{topic_name}");
        let topic = find_or_create_topic(&publisher, &topic_full_name).unwrap();

        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, publisher, topic, utils).expect("writer loop failed")
        });

        Ok(handle)
    }
}

fn connect(endpoint: &str) -> Channel {
    // Set up the gRPC environment.
    let env = Arc::new(EnvBuilder::new().build());
    let creds =
        ChannelCredentials::google_default_credentials().expect("No Google credentials found");

    // Create a channel to connect to Gcloud.
    ChannelBuilder::new(env)
        // Set the max size to correspond to server-side limits.
        .max_send_message_len(1 << 28)
        .max_receive_message_len(1 << 28)
        .secure_connect(endpoint, creds)
}

fn find_or_create_topic(client: &PublisherClient, topic_name: &str) -> grpcio::Result<Topic> {
    // find topic
    let mut request = GetTopicRequest::new();

    request.set_topic(topic_name.to_string());

    if let Ok(topic) = client.get_topic(&request) {
        return Ok(topic);
    }

    // otherwise create topic
    let mut labels = HashMap::new();

    // TODO: do we need this?
    labels.insert("environment".to_string(), "test".to_string());

    let mut topic = Topic::new();

    topic.set_name(topic_name.to_string());
    topic.set_labels(labels);

    client.create_topic(&topic)
}
