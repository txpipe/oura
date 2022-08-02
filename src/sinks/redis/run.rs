use super::StreamStrategy;
use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};
use serde_json::json;
use std::sync::Arc;

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
        let key = key(&event);

        let stream = match stream_strategy {
            StreamStrategy::ByEventType => event.data.clone().to_string().to_lowercase(),
            _ => redis_stream.clone(),
        };

        log::debug!(
            "Stream: {:?}, Key: {:?}, Event: {:?}",
            &stream,
            &key,
            &event
        );

        let result: Result<(), _> = redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg(&[(key, json!(event).to_string())])
            .query(conn);

        match result {
            Ok(_) => {
                utils.track_sink_progress(&event);
            }
            Err(err) => {
                log::error!("error sending message to redis: {}", err);
                return Err(Box::new(err));
            }
        }
    }

    Ok(())
}
