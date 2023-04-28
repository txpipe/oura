use std::time::Duration;

use gasket::{messaging::SendPort, runtime::Tether};
use pallas::upstream::UpstreamEvent;
use serde::Deserialize;

use crate::framework::*;

pub type Adapter = gasket::messaging::tokio::MapSendAdapter<UpstreamEvent, ChainEvent>;
pub type PallasStage = pallas::upstream::n2n::Stage<Cursor, Adapter>;
pub type PallasWorker = pallas::upstream::n2n::Worker<Cursor, Adapter>;

pub struct Stage(PallasStage);

impl Stage {
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
        let tether = gasket::runtime::spawn_stage::<PallasWorker>(self.0);

        Ok(vec![tether])
    }
}

#[derive(Deserialize)]
pub struct Config {
    peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = PallasStage::new(
            self.peers
                .first()
                .cloned()
                .ok_or_else(|| Error::config("at least one upstream peer is required"))?,
            ctx.chain.magic,
            ctx.cursor.clone(),
            gasket::runtime::Policy {
                bootstrap_retry: ctx.retries.clone(),
                work_retry: ctx.retries.clone(),
                ..Default::default()
            },
        );

        Ok(Stage(stage))
    }
}
