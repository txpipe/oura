use aws_sdk_lambda::{types::Blob, Client};
use serde_json::json;
use std::sync::Arc;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

async fn invoke_lambda_function(
    client: Arc<Client>,
    function_name: &str,
    event: &Event,
) -> Result<(), Error> {
    let body = json!(event).to_string();

    let req = client
        .invoke()
        .function_name(function_name)
        .payload(Blob::new(body));

    let res = req.send().await?;

    log::trace!("Lambda invoke response: {:?}", res);

    Ok(())
}

pub fn writer_loop(
    input: StageReceiver,
    client: Client,
    function_name: &str,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    for event in input.iter() {
        let client = client.clone();

        let result = rt.block_on(invoke_lambda_function(client, function_name, &event));

        match result {
            Ok(_) => {
                // notify the pipeline where we are
                utils.track_sink_progress(&event);
            }
            Err(err) => {
                log::error!("unrecoverable error invoking lambda function: {:?}", err);
                return Err(err);
            }
        }
    }

    Ok(())
}
