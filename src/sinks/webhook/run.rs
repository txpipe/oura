use std::{sync::Arc, time::Duration};

use reqwest::blocking::Client;
use serde::Serialize;

use crate::{model::Event, pipelining::StageReceiver, utils::Utils, Error};

use super::ErrorPolicy;

#[derive(Serialize)]
struct RequestBody {
    #[serde(flatten)]
    event: Event,
    variant: String,
    timestamp: Option<u64>,
}

impl From<Event> for RequestBody {
    fn from(event: Event) -> Self {
        let timestamp = event.context.timestamp.map(|x| x * 1000);
        let variant = event.data.to_string();

        RequestBody {
            event,
            timestamp,
            variant,
        }
    }
}

fn execute_fallible_request(
    client: &Client,
    url: &str,
    body: &RequestBody,
    policy: &ErrorPolicy,
    retry_quota: usize,
    backoff_delay: Duration,
) -> Result<(), Error> {
    let request = client.post(url).json(body).build()?;

    let result = client
        .execute(request)
        .and_then(|res| res.error_for_status());

    match (result, policy, retry_quota) {
        (Ok(_), _, _) => {
            log::info!("successful http request to webhook");
            Ok(())
        }
        (Err(x), ErrorPolicy::Exit, 0) => Err(x.into()),
        (Err(x), ErrorPolicy::Continue, 0) => {
            log::warn!("failed to send webhook request: {:?}", x);
            Ok(())
        }
        (Err(x), _, quota) => {
            log::warn!("failed attempt to execute webhook request: {:?}", x);
            std::thread::sleep(backoff_delay);
            execute_fallible_request(client, url, body, policy, quota - 1, backoff_delay)
        }
    }
}

pub(crate) fn request_loop(
    input: StageReceiver,
    client: &Client,
    url: &str,
    error_policy: &ErrorPolicy,
    max_retries: usize,
    backoff_delay: Duration,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    loop {
        let event = input.recv().unwrap();

        // notify progress to the pipeline
        utils.track_sink_progress(&event);

        let body = RequestBody::from(event);

        execute_fallible_request(client, url, &body, error_policy, max_retries, backoff_delay)?;
    }
}
