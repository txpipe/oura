use redis::{Client};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::*;

#[derive(Debug, Clone, Deserialize)]
pub enum StreamConfig {
    MultiStream,
    SingleStream
}   

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis_server      : String,
    pub stream_config     : StreamConfig,
    pub stream_name       : Option<String>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = Client::open(self.inner.redis_server.clone())?;
        let mut connection = client.get_connection()?;
        log::debug!("Connected to Redis Database!");
        let stream_config = self.inner.stream_config.clone();
        let redis_stream = self.inner.stream_name.clone().unwrap_or("oura".to_string());

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            producer_loop(input, utils, &mut connection, stream_config, redis_stream).expect("redis sink loop failed");
        });

        Ok(handle)
    }
}
