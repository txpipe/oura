use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    sinks::common::web::{build_headers_map, request_loop, ErrorPolicy, APP_USER_AGENT},
    utils::{retry, WithUtils},
};

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub authorization: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,

    /// Accept invalid TLS certificates
    ///
    /// DANGER Will Robinson! Set this flag to skip TLS verification. Main
    /// use-case for this flag is to allow self-signed certificates. Beware that
    /// other invalid properties will be ommited too, such as expiration date.
    pub allow_invalid_certs: Option<bool>,

    pub error_policy: Option<ErrorPolicy>,
    pub retry_policy: Option<retry::Policy>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(build_headers_map(
                self.inner.authorization.as_ref(),
                self.inner.headers.as_ref(),
            )?)
            .danger_accept_invalid_certs(self.inner.allow_invalid_certs.unwrap_or(false))
            .timeout(Duration::from_millis(self.inner.timeout.unwrap_or(30000)))
            .build()?;

        let url = self.inner.url.clone();

        let error_policy = self
            .inner
            .error_policy
            .as_ref()
            .cloned()
            .unwrap_or(ErrorPolicy::Exit);

        let retry_policy = self.inner.retry_policy.unwrap_or_default();

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            request_loop(input, &client, &url, &error_policy, &retry_policy, utils)
                .expect("request loop failed")
        });

        Ok(handle)
    }
}
