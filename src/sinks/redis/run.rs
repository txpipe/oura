#![allow(unused_variables)]
use super::StreamStrategy;
use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Serialize)]
pub struct RedisRecord {
    pub event: Event,
    pub key: String,
}

impl From<Event> for RedisRecord {
    fn from(event: Event) -> Self {
        let key = key(&event);
        RedisRecord { event, key }
    }
}

fn key(event: &Event) -> String {
    if let Some(fingerprint) = &event.fingerprint {
        fingerprint.clone()
    } else {
        event.data.clone().to_string().to_lowercase()
    }
}

pub fn producer_loop(
    input: StageReceiver,
    utils: Arc<Utils>,
    conn: &mut redis::Connection,
    stream_strategy: StreamStrategy,
    redis_stream: String,
) -> Result<(), Error> {
    for event in input.iter() {
        utils.track_sink_progress(&event);
        let payload = RedisRecord::from(event);

        let stream = match stream_strategy {
            StreamStrategy::ByEventType => payload.event.data.clone().to_string().to_lowercase(),
            _ => redis_stream.clone(),
        };

        log::debug!(
            "Stream: {:?}, Key: {:?}, Event: {:?}",
            stream,
            payload.key,
            payload.event
        );

        let _: () = redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg(&[(payload.key, json!(payload.event).to_string())])
            .query(conn)?;
    }

    Ok(())
}
