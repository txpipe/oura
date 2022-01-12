use std::time::Duration;

use reqwest::blocking::{Client, Request};
use serde::Serialize;

use crate::framework::{Error, Event, StageReceiver};

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
    request: Request,
    policy: &ErrorPolicy,
    retry_quota: usize,
    backoff_delay: &Duration,
) -> Result<(), Error> {
    let result = client.execute(request);

    match (result, policy, retry_quota) {
        (Err(x), ErrorPolicy::Retry, 0) => Err("max retries reached".into()),
        (Err(x), ErrorPolicy::Retry, quota) => {
            std::thread::sleep(backoff_delay.clone());
            execute_fallible_request(client, request, policy, retry_quota - 1, backoff_delay)
        }
        (Err(x), ErrorPolicy::Continue, _) => Ok(()),
        (Err(x), ErrorPolicy::Exit, _) => Err(x.into()),
        (Ok(_), _, _) => Ok(()),
    }
}

pub(crate) fn request_loop(
    input: StageReceiver,
    client: &Client,
    url: &str,
    error_policy: &ErrorPolicy,
    max_retries: usize,
    backoff_delay: &Duration,
) {
    loop {
        let event = input.recv().unwrap();
        let client = client.clone();
        let body = RequestBody::from(event);

        let request = client.post(url).json(&body).build().unwrap();
        execute_fallible_request(&client, request, error_policy, max_retries, &backoff_delay).unwrap();
    }
}
