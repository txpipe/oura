use std::borrow::Cow;

use aws_config::{self, meta::region::RegionProviderChain, RetryConfig};
use aws_sdk_sqs::{Client, Region};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::writer_loop;

const DEFAULT_MAX_RETRIES: u32 = 5;

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub region: String,
    pub queue_url: String,
    pub fifo: Option<bool>,
    pub group_id: Option<String>,
    pub max_retries: Option<u32>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let explicit_region = Cow::Owned(self.inner.region.to_owned());

        let region_provider =
            RegionProviderChain::first_try(Region::new(explicit_region)).or_default_provider();

        let aws_config = tokio::runtime::Runtime::new()?
            .block_on(aws_config::from_env().region(region_provider).load());

        let retry_config = RetryConfig::new()
            .with_max_attempts(self.inner.max_retries.unwrap_or(DEFAULT_MAX_RETRIES));

        let sqs_config = aws_sdk_sqs::config::Builder::from(&aws_config)
            .retry_config(retry_config)
            .build();

        let client = Client::from_conf(sqs_config);
        let queue_url = self.inner.queue_url.clone();
        let fifo = self.inner.fifo.unwrap_or_default();
        let group_id = self
            .inner
            .group_id
            .clone()
            .unwrap_or_else(|| "oura-sink".to_string());

        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, client, &queue_url, fifo, &group_id, utils)
                .expect("writer loop failed")
        });

        Ok(handle)
    }
}
