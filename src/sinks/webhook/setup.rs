use std::{collections::HashMap, time::Duration};

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde_derive::Deserialize;

use crate::framework::{BootstrapResult, Error, SinkConfig, StageReceiver};

use super::run::request_loop;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
pub enum ErrorPolicy {
    Retry,
    Continue,
    Exit,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    url: String,
    authorization: Option<String>,
    headers: HashMap<String, String>,
    timeout: Option<u64>,
    error_policy: Option<ErrorPolicy>,
    max_retries: Option<usize>,
    backoff_delay: Option<u64>,
}

fn build_headers_map(config: &Config) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();

    for (name, value) in config.headers.iter() {
        let name = HeaderName::try_from(name)?;
        let value = HeaderValue::try_from(value)?;
        headers.insert(name, value);
    }

    if let Some(auth_value) = config.authorization {
        let auth_value = HeaderValue::try_from(auth_value)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    Ok(headers)
}

impl SinkConfig for Config {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(build_headers_map(self)?)
            .timeout(Duration::from_millis(self.timeout.unwrap_or(30000)))
            .build();

        let handle = std::thread::spawn(move || {
            request_loop(input, client, url, &self.error_policy).expect("request loop failed")
        });

        Ok(handle)
    }
}
