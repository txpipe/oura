use kafka::producer::{Producer, Record};
use log::debug;

use crate::{
    framework::{Error, Event},
    pipelining::StageReceiver,
};

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
) -> Result<(), Error> {
    loop {
        let evt = input.recv()?;
        let json = serde_json::to_vec(&evt)?;
        let key = define_event_key(&evt, &partitioning);

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

        debug!("pushed event to kafka: {:?}", &evt);
    }
}
