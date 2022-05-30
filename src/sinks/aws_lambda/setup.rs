use aws_sdk_lambda::{Client, Region, RetryConfig};
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
    pub function_name: String,
    pub max_retries: Option<u32>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let explicit_region = self.inner.region.to_owned();

        let aws_config = tokio::runtime::Builder::new_current_thread()
            .build()?
            .block_on(
                aws_config::from_env()
                    .region(Region::new(explicit_region))
                    .load(),
            );

        let retry_config = RetryConfig::new()
            .with_max_attempts(self.inner.max_retries.unwrap_or(DEFAULT_MAX_RETRIES));

        let lambda_config = aws_sdk_lambda::config::Builder::from(&aws_config)
            .retry_config(retry_config)
            .build();

        let client = Client::from_conf(lambda_config);
        let function_name = self.inner.function_name.clone();

        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, client, &function_name, utils).expect("writer loop failed")
        });

        Ok(handle)
    }
}
