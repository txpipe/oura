use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    sinks::ErrorPolicy,
    utils::{retry, WithUtils},
};

use super::run::writer_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub topic: String,
    pub credentials: String,
    pub error_policy: Option<ErrorPolicy>,
    pub retry_policy: Option<retry::Policy>,
}

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

        let retry_policy = self.inner.retry_policy.unwrap_or_default();

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            writer_loop(
                input,
                credentials,
                topic_name,
                &error_policy,
                &retry_policy,
                utils,
            )
            .expect("writer loop failed");
        });

        Ok(handle)
    }
}
