use std::sync::Arc;
use redis::{Commands, Connection};
use log::debug;
use serde_json::json;
use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

use super::PartitionStrategy;

fn define_event_key(event: &Event, strategy: &PartitionStrategy) -> String {
    match strategy {
        PartitionStrategy::ByBlock => format!("{:?}",event.context.block_number),
        PartitionStrategy::Timestamp => "*".to_string(),
    }
}

fn write_redis(
    redis: &mut Connection,
    stream: String,
    key: String,
    key_values: &[(&str, &str)],
) -> Result<String, Error> {
    let id = redis.xadd::<&str, &str, &str, &str, String>(&stream, &key, key_values)?;
    Ok(id)
}

pub fn producer_loop(
    input: StageReceiver,
    redis: &mut Connection,
    stream: String,
    partitioning: PartitionStrategy,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    
    for event in input.iter() {
        // notify the pipeline where we are
        utils.track_sink_progress(&event);
        let tstream = stream.clone();
        let json: &str = &json!(event).to_string();
        let key_values = &[("json",json)];
        let key = define_event_key(&event, &partitioning);
        let _ = write_redis(redis, tstream, key, key_values);

        debug!("pushed event to redis: {:?}", &event);
    }
    Ok(())
}
  
