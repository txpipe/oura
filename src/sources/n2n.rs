use gasket::messaging::SendPort;
use pallas::upstream::UpstreamEvent;
use serde::Deserialize;

use crate::framework::*;

pub type Adapter = gasket::messaging::tokio::MapSendAdapter<UpstreamEvent, ChainEvent>;
pub type Stage = pallas::upstream::n2n::Stage<Cursor, Adapter>;

pub fn connect_output(stage: &mut Stage, adapter: OutputAdapter) {
    let adapter = gasket::messaging::tokio::MapSendAdapter::new(adapter, |x| match x {
        UpstreamEvent::RollForward(slot, hash, body) => Some(ChainEvent::Apply(
            pallas::network::miniprotocols::Point::Specific(slot, hash.to_vec()),
            Record::CborBlock(body),
        )),
        UpstreamEvent::Rollback(x) => Some(ChainEvent::Reset(x)),
    });

    stage.downstream_port().connect(adapter);
}

#[derive(Deserialize)]
pub struct Config {
    peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage::new(
            self.peers
                .first()
                .cloned()
                .ok_or_else(|| Error::config("at least one upstream peer is required"))?,
            ctx.chain.magic,
            ctx.cursor.clone(),
        );

        Ok(stage)
    }
}
