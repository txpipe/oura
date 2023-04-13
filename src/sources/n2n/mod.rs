use std::time::Duration;

use gasket::{
    messaging::SendPort,
    runtime::{Policy, Tether},
};
use pallas::upstream::{n2n::Worker, UpstreamEvent};
use serde::Deserialize;

use crate::framework::*;

pub type Adapter = gasket::messaging::tokio::MapSendAdapter<UpstreamEvent, ChainEvent>;

pub struct Bootstrapper(Worker<Cursor, Adapter>);

impl Bootstrapper {
    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        let adapter = gasket::messaging::tokio::MapSendAdapter::new(adapter, |x| match x {
            UpstreamEvent::RollForward(slot, hash, body) => Some(ChainEvent::Apply(
                pallas::network::miniprotocols::Point::Specific(slot, hash.to_vec()),
                Record::CborBlock(body),
            )),
            UpstreamEvent::Rollback(x) => Some(ChainEvent::Reset(x)),
        });

        self.0.downstream_port().connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let retry_policy = gasket::retries::Policy {
            max_retries: 10,
            backoff_unit: Duration::from_secs(2),
            backoff_factor: 2,
            max_backoff: Duration::from_secs(30),
            dismissible: false,
        };

        let tether = gasket::runtime::spawn_stage(
            self.0,
            Policy {
                work_retry: retry_policy.clone(),
                bootstrap_retry: retry_policy,
                ..Default::default()
            },
            Some("source"),
        );

        Ok(vec![tether])
    }
}

#[derive(Deserialize)]
pub struct Config {
    peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker::<_, Adapter>::new(
            self.peers
                .first()
                .cloned()
                .ok_or_else(|| Error::config("at least one upstream peer is required"))?,
            ctx.chain.magic,
            ctx.cursor.clone(),
        );

        Ok(Bootstrapper(worker))
    }
}
