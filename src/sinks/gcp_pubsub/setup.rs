use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::writer_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub topic: String,
    pub credentials: String,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let credentials = self.inner.credentials.to_owned();
        let topic_name = self.inner.topic.to_owned();

        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, credentials, topic_name, utils).expect("writer loop failed");
        });

        Ok(handle)
    }
}
