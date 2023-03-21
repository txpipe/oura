use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod deno;
pub mod dsl;
pub mod json;
pub mod legacy_v1;
pub mod noop;
pub mod wasm;

pub enum Bootstrapper {
    Noop(noop::Bootstrapper),
    Dsl(dsl::Bootstrapper),
    Json(json::Bootstrapper),
    LegacyV1(legacy_v1::Bootstrapper),
    Wasm(wasm::Bootstrapper),
    Deno(deno::Bootstrapper),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.connect_input(adapter),
            Bootstrapper::Dsl(p) => p.connect_input(adapter),
            Bootstrapper::Json(p) => p.connect_input(adapter),
            Bootstrapper::LegacyV1(p) => p.connect_input(adapter),
            Bootstrapper::Wasm(p) => p.connect_input(adapter),
            Bootstrapper::Deno(p) => p.connect_input(adapter),
        }
    }

    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.connect_output(adapter),
            Bootstrapper::Dsl(p) => p.connect_output(adapter),
            Bootstrapper::Json(p) => p.connect_output(adapter),
            Bootstrapper::LegacyV1(p) => p.connect_output(adapter),
            Bootstrapper::Wasm(p) => p.connect_output(adapter),
            Bootstrapper::Deno(p) => p.connect_output(adapter),
        }
    }

    fn spawn(self) -> Result<Vec<Tether>, Error> {
        match self {
            Bootstrapper::Noop(x) => x.spawn(),
            Bootstrapper::Dsl(x) => x.spawn(),
            Bootstrapper::Json(x) => x.spawn(),
            Bootstrapper::LegacyV1(x) => x.spawn(),
            Bootstrapper::Wasm(x) => x.spawn(),
            Bootstrapper::Deno(x) => x.spawn(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Noop(noop::Config),
    Dsl(dsl::Config),
    Json(json::Config),
    LegacyV1(legacy_v1::Config),
    Wasm(wasm::Config),
    Deno(deno::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
            Config::Dsl(c) => Ok(Bootstrapper::Dsl(c.bootstrapper(ctx)?)),
            Config::Json(c) => Ok(Bootstrapper::Json(c.bootstrapper(ctx)?)),
            Config::LegacyV1(c) => Ok(Bootstrapper::LegacyV1(c.bootstrapper(ctx)?)),
            Config::Wasm(c) => Ok(Bootstrapper::Wasm(c.bootstrapper(ctx)?)),
            Config::Deno(c) => Ok(Bootstrapper::Deno(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::LegacyV1(Default::default())
    }
}
