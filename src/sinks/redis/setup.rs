use redis::{Client};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::*;

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub redis_server      : String,
    pub redis_stream      : Option<String>,
    pub strategy          : Option<String>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        // redis://[<username>][:<password>@]<hostname>[:port][/<db>]
        let client = Client::open(self.inner.redis_server.clone())?;
        let mut connection = client.get_connection()?;
        log::info!("Connected to Redis Database!");

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            producer_loop(input, utils, &mut connection).expect("redis sink loop failed");
        });

        Ok(handle)
    }
}
