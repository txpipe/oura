use lapin::{Connection, ConnectionProperties};
use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::{retry, WithUtils},
};

use super::run::publisher_loop;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub uri: String,
    pub exchange: String,
    pub routing_key: Option<String>,
    pub retry_policy: Option<retry::Policy>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let rt = &tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let connection = rt
            .block_on(Connection::connect(
                &self.inner.uri,
                ConnectionProperties::default(),
            ))
            .expect("error to connect");

        connection.on_error(|err| {
            log::error!("{}", err);
            std::process::exit(1);
        });

        let exchange = self.inner.exchange.to_owned();
        let routing_key = self.inner.routing_key.to_owned().unwrap_or("".to_string());

        let retry_policy = self.inner.retry_policy.unwrap_or_default();
        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || {
            publisher_loop(
                input,
                connection,
                exchange,
                routing_key,
                retry_policy,
                utils,
            )
            .expect("publisher loop failed");
        });

        Ok(handle)
    }
}
