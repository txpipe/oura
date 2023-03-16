use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod dsl;
pub mod noop;

pub enum Bootstrapper {
    Noop(noop::Bootstrapper),
    Dsl(dsl::Bootstrapper),
    //Wasm,
}

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: FilterInputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.connect_input(adapter),
            Bootstrapper::Dsl(p) => p.connect_input(adapter),
        }
    }

    pub fn connect_output(&mut self, adapter: FilterOutputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.connect_output(adapter),
            Bootstrapper::Dsl(p) => p.connect_output(adapter),
        }
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        match self {
            Bootstrapper::Noop(x) => x.spawn(),
            Bootstrapper::Dsl(x) => x.spawn(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Noop(noop::Config),
    Dsl(dsl::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
            Config::Dsl(c) => Ok(Bootstrapper::Dsl(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::Noop(Default::default())
    }
}
