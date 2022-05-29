use redis::{Connection};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::producer_loop;

#[derive(Debug, Deserialize, Clone)]
pub enum PartitionStrategy {
    ByBlock,
    Timestamp,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub stream: String,
    pub paritioning: Option<PartitionStrategy>,
}

pub fn redis_connection(u: String) -> Connection {
    redis::Client::open(u)
      .expect("failed to open redis client")
      .get_connection()
      .expect("failed to get redis connection")
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        
        let mut redis = redis_connection(self.inner.url.clone());
        
        let stream = self.inner.stream.clone();
        let utils = self.utils.clone();

        let partitioning = self
            .inner
            .paritioning
            .clone()
            .unwrap_or(PartitionStrategy::Timestamp);

        let handle = std::thread::spawn(move || {
            producer_loop(input, &mut redis, stream, partitioning, utils)
                .expect("producer loop failed")
        });

        Ok(handle)
    }
}
