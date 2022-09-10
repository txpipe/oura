use std::time::Duration;

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::reducer_loop;

const THROTTLE_MIN_SPAN_MILLIS: u64 = 300;

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub throttle_min_span_millis: Option<u64>,
    pub wrap: Option<bool>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let throttle_min_span = Duration::from_millis(
            self.inner
                .throttle_min_span_millis
                .unwrap_or(THROTTLE_MIN_SPAN_MILLIS),
        );

        let wrap = self.inner.wrap.unwrap_or(false);
        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            reducer_loop(throttle_min_span, wrap, input, utils).expect("terminal sink loop failed");
        });

        Ok(handle)
    }
}
