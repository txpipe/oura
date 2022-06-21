use std::{collections::HashMap, sync::Arc};

use reqwest::{
    blocking::Client,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};

use crate::{
    model::Event,
    pipelining::StageReceiver,
    utils::{retry, Utils},
    Error,
};

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Clone)]
pub enum ErrorPolicy {
    Continue,
    Exit,
}

pub fn build_headers_map(
    authorization: Option<&String>,
    extra: Option<&HashMap<String, String>>,
) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::try_from("application/json")?,
    );

    if let Some(auth_value) = &authorization {
        let auth_value = HeaderValue::try_from(*auth_value)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    if let Some(custom) = &extra {
        for (name, value) in custom.iter() {
            let name = HeaderName::try_from(name)?;
            let value = HeaderValue::try_from(value)?;
            headers.insert(name, value);
        }
    }

    Ok(headers)
}

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

fn execute_fallible_request(client: &Client, url: &str, body: &RequestBody) -> Result<(), Error> {
    let request = client.post(url).json(body).build()?;

    client
        .execute(request)
        .and_then(|res| res.error_for_status())?;

    Ok(())
}

pub(crate) fn request_loop(
    input: StageReceiver,
    client: &Client,
    url: &str,
    error_policy: &ErrorPolicy,
    retry_policy: &retry::Policy,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    for event in input.iter() {
        // notify progress to the pipeline
        utils.track_sink_progress(&event);

        let body = RequestBody::from(event);

        let result = retry::retry_operation(
            || execute_fallible_request(client, url, &body),
            retry_policy,
        );

        match result {
            Ok(()) => (),
            Err(err) => match error_policy {
                ErrorPolicy::Exit => return Err(err),
                ErrorPolicy::Continue => {
                    log::warn!("failed to send webhook request: {:?}", err);
                }
            },
        }
    }

    Ok(())
}
