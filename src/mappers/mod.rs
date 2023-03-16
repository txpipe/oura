use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod json;
pub mod legacy_v1;
pub mod wasm;

pub enum Bootstrapper {
    Json(json::Bootstrapper),
    LegacyV1(legacy_v1::Bootstrapper),
    Wasm(wasm::Bootstrapper),
}

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: MapperInputAdapter) {
        match self {
            Bootstrapper::Json(p) => p.connect_input(adapter),
            Bootstrapper::LegacyV1(p) => p.connect_input(adapter),
            Bootstrapper::Wasm(p) => p.connect_input(adapter),
        }
    }

    pub fn connect_output(&mut self, adapter: MapperOutputAdapter) {
        match self {
            Bootstrapper::Json(p) => p.connect_output(adapter),
            Bootstrapper::LegacyV1(p) => p.connect_output(adapter),
            Bootstrapper::Wasm(p) => p.connect_output(adapter),
        }
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        match self {
            Bootstrapper::Json(x) => x.spawn(),
            Bootstrapper::LegacyV1(x) => x.spawn(),
            Bootstrapper::Wasm(x) => x.spawn(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Json(json::Config),
    LegacyV1(legacy_v1::Config),
    Wasm(wasm::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Json(c) => Ok(Bootstrapper::Json(c.bootstrapper(ctx)?)),
            Config::LegacyV1(c) => Ok(Bootstrapper::LegacyV1(c.bootstrapper(ctx)?)),
            Config::Wasm(c) => Ok(Bootstrapper::Wasm(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::LegacyV1(Default::default())
    }
}
