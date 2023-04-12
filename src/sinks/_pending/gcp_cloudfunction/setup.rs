use std::time::Duration;

use serde::Deserialize;

use crate::{
    framework::{BootstrapResult, SinkProvider, StageReceiver},
    sinks::common::web::{build_headers_map, request_loop, ErrorPolicy, APP_USER_AGENT},
    utils::{retry, WithUtils},
};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub name: String,
    pub project_id: String,
    pub region: String,
    pub timeout: Option<u64>,
    pub authorization: Option<String>,
    pub error_policy: Option<ErrorPolicy>,
    pub retry_policy: Option<retry::Policy>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(build_headers_map(self.inner.authorization.as_ref(), None)?)
            .timeout(Duration::from_millis(self.inner.timeout.unwrap_or(30000)))
            .build()?;

        let url = format!(
            "https://{}-{}.cloudfunctions.net/{}",
            self.inner.region, self.inner.project_id, self.inner.name,
        );

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
