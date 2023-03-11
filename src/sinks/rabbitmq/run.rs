use std::sync::Arc;

use lapin::{options::BasicPublishOptions, BasicProperties, Channel, Connection};
use serde_json::json;

use crate::{
    model::Event,
    pipelining::StageReceiver,
    utils::{retry, Utils},
    Error,
};

async fn publish_message(
    channel: &Channel,
    exchange: &str,
    routing_key: &str,
    event: &Event,
) -> Result<(), crate::Error> {
    let body = json!(event).to_string();

    channel
        .basic_publish(
            exchange,
            routing_key,
            BasicPublishOptions::default(),
            body.as_bytes(),
            BasicProperties::default(),
        )
        .await?;

    Ok(())
}

pub fn publisher_loop(
    input: StageReceiver,
    connection: Connection,
    exchange: String,
    routing_key: String,
    retry_policy: retry::Policy,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let rt = &tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let channel = rt.block_on(connection.create_channel())?;

    for event in input.iter() {
        let result = retry::retry_operation(
            || rt.block_on(publish_message(&channel, &exchange, &routing_key, &event)),
            &retry_policy,
        );

        match result {
            Ok(_) => {
                log::debug!("pushed event to rabbitmq: {:?}", &event);
                utils.track_sink_progress(&event);
            }
            Err(err) => {
                log::error!("error sending rabbitmq message: {}", err);
                return Err(err);
            }
        }
    }

    Ok(())
}
