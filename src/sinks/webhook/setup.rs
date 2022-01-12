use std::{collections::HashMap, time::Duration};

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde_derive::Deserialize;

use crate::framework::{BootstrapResult, Error, SinkConfig, StageReceiver};

use super::run::request_loop;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Clone)]
pub enum ErrorPolicy {
    Continue,
    Exit,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    url: String,
    authorization: Option<String>,
    headers: Option<HashMap<String, String>>,
    timeout: Option<u64>,
    error_policy: Option<ErrorPolicy>,
    max_retries: Option<usize>,
    backoff_delay: Option<u64>,
}

fn build_headers_map(config: &Config) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::try_from("application/json")?,
    );

    if let Some(auth_value) = &config.authorization {
        let auth_value = HeaderValue::try_from(auth_value)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    if let Some(custom) = &config.headers {
        for (name, value) in custom.iter() {
            let name = HeaderName::try_from(name)?;
            let value = HeaderValue::try_from(value)?;
            headers.insert(name, value);
        }
    }

    Ok(headers)
}

const DEFAULT_MAX_RETRIES: usize = 20;
const DEFAULT_BACKOFF_DELAY: u64 = 5_000;

impl SinkConfig for Config {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(build_headers_map(self)?)
            .timeout(Duration::from_millis(self.timeout.unwrap_or(30000)))
            .build()?;

        let url = self.url.clone();

        let error_policy = self
            .error_policy
            .as_ref()
            .cloned()
            .unwrap_or(ErrorPolicy::Exit);

        let max_retries = self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES);

        let backoff_delay =
            Duration::from_millis(self.backoff_delay.unwrap_or(DEFAULT_BACKOFF_DELAY));

        let handle = std::thread::spawn(move || {
            request_loop(
                input,
                &client,
                &url,
                &error_policy,
                max_retries,
                backoff_delay,
            )
            .expect("request loop failed")
        });

        Ok(handle)
    }
}
