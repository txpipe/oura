use std::sync::Arc;

use kafka::producer::{Producer, Record};
use log::debug;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

use super::PartitionStrategy;

fn define_event_key(event: &Event, strategy: &PartitionStrategy) -> Option<[u8; 8]> {
    match strategy {
        PartitionStrategy::ByBlock => event.context.block_number.map(|n| n.to_be_bytes()),
        PartitionStrategy::Random => None,
    }
}

pub fn producer_loop(
    input: StageReceiver,
    mut producer: Producer,
    topic: String,
    partitioning: PartitionStrategy,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    loop {
        let event = input.recv()?;

        // notify the pipeline where we are
        utils.track_sink_progress(&event);

        let json = serde_json::to_vec(&event)?;
        let key = define_event_key(&event, &partitioning);

        match key {
            Some(key) => {
                let r = Record::from_key_value(&topic, &key[..], json);
                producer.send(&r)?;
            }
            None => {
                let r = Record::from_value(&topic, json);
                producer.send(&r)?;
            }
        };

        debug!("pushed event to kafka: {:?}", &event);
    }
}
