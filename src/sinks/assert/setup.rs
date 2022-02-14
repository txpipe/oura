use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::assertion_loop;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub break_on_failure: bool,

    #[serde(default = "Vec::new")]
    pub skip_assertions: Vec<String>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let utils = self.utils.clone();

        let config = self.inner.clone();

        let handle = std::thread::spawn(move || {
            assertion_loop(input, config, utils).expect("assertion loop failed")
        });

        Ok(handle)
    }
}
