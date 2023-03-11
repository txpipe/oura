use pallas::network::upstream::n2n::{
    Bootstrapper as PallasBootstrapper, Runtime as PallasRuntime,
};
use serde::Deserialize;

use crate::framework::*;

pub struct Runtime(PallasRuntime);

pub struct Bootstrapper(PallasBootstrapper);

impl Bootstrapper {
    pub fn borrow_output_port(&mut self) -> &mut SourceOutputPort {
        self.borrow_output_port()
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        let upstream = self.0.spawn().map_err(Error::custom)?;
        Ok(Runtime(upstream))
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
