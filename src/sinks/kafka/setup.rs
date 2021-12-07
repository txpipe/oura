use std::{sync::mpsc::Receiver, time::Duration};

use kafka::{client::RequiredAcks, producer::Producer};
use serde_derive::Deserialize;

use crate::{
    framework::{BootstrapResult, SinkConfig},
    ports::Event,
};

use super::run::producer_loop;

#[derive(Debug, Deserialize, Clone)]
pub enum PartitionStrategy {
    ByBlock,
    Random,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    brokers: Vec<String>,
    topic: String,
    ack_timeout_secs: Option<u64>,
    paritioning: Option<PartitionStrategy>,
}

impl SinkConfig for Config {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult {
        let mut builder = Producer::from_hosts(self.brokers.clone());

        if let Some(timeout) = &self.ack_timeout_secs {
            builder = builder
                .with_ack_timeout(Duration::from_secs(*timeout))
                .with_required_acks(RequiredAcks::One)
        };

        let producer = builder.create()?;
        let topic = self.topic.clone();
        let partitioning = self
            .paritioning
            .clone()
            .unwrap_or(PartitionStrategy::Random);
        let handle = std::thread::spawn(move || {
            producer_loop(input, producer, topic, partitioning).unwrap()
        });

        Ok(handle)
    }
}
