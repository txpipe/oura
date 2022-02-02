use std::{collections::HashMap, time::Duration};

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
    Error,
};

use super::run::request_loop;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Clone)]
pub enum ErrorPolicy {
    Continue,
    Exit,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub authorization: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,
    pub error_policy: Option<ErrorPolicy>,
    pub max_retries: Option<usize>,
    pub backoff_delay: Option<u64>,
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

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(build_headers_map(&self.inner)?)
            .timeout(Duration::from_millis(self.inner.timeout.unwrap_or(30000)))
            .build()?;

        let url = self.inner.url.clone();

        let error_policy = self
            .inner
            .error_policy
            .as_ref()
            .cloned()
            .unwrap_or(ErrorPolicy::Exit);

        let max_retries = self.inner.max_retries.unwrap_or(DEFAULT_MAX_RETRIES);

        let backoff_delay =
            Duration::from_millis(self.inner.backoff_delay.unwrap_or(DEFAULT_BACKOFF_DELAY));

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            request_loop(
                input,
                &client,
                &url,
                &error_policy,
                max_retries,
                backoff_delay,
                utils,
            )
            .expect("request loop failed")
        });

        Ok(handle)
    }
}
