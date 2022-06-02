use redis::{Client};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::*;

#[derive(Debug, Clone, Deserialize)]
pub enum StreamStrategy {
    ByEventType,
    None
}   

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis_server      : String,
    pub stream_strategy   : Option<StreamStrategy>,
    pub stream_name       : Option<String>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let client = Client::open(self.inner.redis_server.clone())?;
        let mut connection = client.get_connection()?;
        log::debug!("Connected to Redis Database!");
        let stream_strategy = match self.inner.stream_strategy.clone() {
            Some(strategy)  => { 
                strategy
            },
            _               => StreamStrategy::None
        };
        let redis_stream = self.inner.stream_name.clone().unwrap_or("oura".to_string());
        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            producer_loop(input, utils, &mut connection, stream_strategy, redis_stream).expect("redis sink loop failed");
        });
        Ok(handle)
    }
}
