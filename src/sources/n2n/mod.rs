use gasket::{messaging::SendPort, runtime::Tether};
use pallas::network::upstream::n2n::{
    Bootstrapper as PallasBootstrapper, Runtime as PallasRuntime,
};
use serde::Deserialize;

use crate::framework::*;

pub struct Runtime(PallasRuntime);

pub struct Bootstrapper(PallasBootstrapper<ChainEvent>);

impl Bootstrapper {
    pub fn connect_output(&mut self, adapter: SourceOutputAdapter) {
        let adapter = gasket::messaging::MapSendAdapter::new(adapter, |x| match x {
            pallas::network::upstream::BlockFetchEvent::RollForward(slot, hash, body) => {
                Some(ChainEvent::Apply(
                    pallas::network::miniprotocols::Point::Specific(slot, hash.to_vec()),
                    Record::CborBlock(body),
                ))
            }
            pallas::network::upstream::BlockFetchEvent::Rollback(x) => Some(ChainEvent::Reset(x)),
        });

        self.0.borrow_output_port().connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let upstream = self.0.spawn().map_err(Error::custom)?;

        Ok(vec![
            upstream.plexer_tether,
            upstream.chainsync_tether,
            upstream.blockfetch_tether,
        ])
    }
}

#[derive(Deserialize)]
pub struct Config {
    peers: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let inner = PallasBootstrapper::new(
            ctx.cursor.clone(),
            self.peers
                .first()
                .cloned()
                .ok_or_else(|| Error::config("at least one upstream peer is required"))?,
            ctx.chain.magic,
        );

        Ok(Bootstrapper(inner))
    }
}
