use std::time::Duration;

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    sinks::ErrorPolicy,
    utils::WithUtils,
};

use super::run::writer_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub topic: String,
    pub credentials: String,
    pub error_policy: Option<ErrorPolicy>,
    pub max_retries: Option<usize>,
    pub backoff_delay: Option<u64>,
}

const DEFAULT_MAX_RETRIES: usize = 20;
const DEFAULT_BACKOFF_DELAY: u64 = 5_000;

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let credentials = self.inner.credentials.to_owned();
        let topic_name = self.inner.topic.to_owned();

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
            writer_loop(
                input,
                credentials,
                topic_name,
                &error_policy,
                max_retries,
                backoff_delay,
                utils,
            )
            .expect("writer loop failed");
        });

        Ok(handle)
    }
}
