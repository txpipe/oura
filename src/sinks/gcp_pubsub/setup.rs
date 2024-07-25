use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    sinks::common::web::ErrorPolicy,
    sinks::gcp_pubsub::run::GenericKV,
    utils::{retry, WithUtils},
};

use super::run::writer_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub topic: String,
    pub error_policy: Option<ErrorPolicy>,
    pub retry_policy: Option<retry::Policy>,
    pub ordering_key: Option<String>,
    pub attributes: Option<GenericKV>,
    pub emulator: Option<bool>,
    pub emulator_endpoint: Option<String>,
    pub emulator_project_id: Option<String>,

    #[warn(deprecated)]
    pub credentials: Option<String>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let topic_name = self.inner.topic.to_owned();
        let mut use_emulator = self.inner.emulator.unwrap_or(false);
        let emulator_endpoint = self.inner.emulator_endpoint.to_owned();
        let emulator_project_id = self.inner.emulator_project_id.to_owned();
        if use_emulator && (emulator_endpoint.is_none() || emulator_project_id.is_none()) {
            use_emulator = false;
        }

        let error_policy = self
            .inner
            .error_policy
            .as_ref()
            .cloned()
            .unwrap_or(ErrorPolicy::Exit);

        let retry_policy = self.inner.retry_policy.unwrap_or_default();
        let ordering_key = self.inner.ordering_key.to_owned().unwrap_or_default();

        let utils = self.utils.clone();

        let attributes = self.inner.attributes.clone().unwrap_or_default();

        let handle = std::thread::spawn(move || {
            writer_loop(
                input,
                &topic_name,
                &error_policy,
                &retry_policy,
                &ordering_key,
                &attributes,
                use_emulator,
                &emulator_endpoint,
                &emulator_project_id,
                utils,
            )
            .expect("writer loop failed");
        });

        Ok(handle)
    }
}
