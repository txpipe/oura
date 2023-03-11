use aws_sdk_s3::{Client, Region, RetryConfig};
use serde::Deserialize;

use crate::{
    framework::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::writer_loop;

const DEFAULT_MAX_RETRIES: u32 = 5;

#[derive(Deserialize, Debug, Clone)]
pub enum Naming {
    Hash,
    SlotHash,
    BlockHash,
    EpochHash,
    EpochSlotHash,
    EpochBlockHash,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ContentType {
    Cbor,
    CborHex,
}

impl From<&ContentType> for String {
    fn from(other: &ContentType) -> Self {
        match other {
            ContentType::Cbor => "application/cbor".to_string(),
            ContentType::CborHex => "text/plain".to_string(),
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub region: String,
    pub bucket: String,
    pub prefix: Option<String>,
    pub naming: Option<Naming>,
    pub content: Option<ContentType>,
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

        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .retry_config(retry_config)
            .build();

        let client = Client::from_conf(s3_config);
        let bucket = self.inner.bucket.clone();
        let prefix = self.inner.prefix.clone().unwrap_or_default();
        let naming = self.inner.naming.clone().unwrap_or(Naming::Hash);
        let content = self.inner.content.clone().unwrap_or(ContentType::Cbor);
        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            writer_loop(input, client, &bucket, &prefix, naming, content, utils)
                .expect("writer loop failed")
        });

        Ok(handle)
    }
}
