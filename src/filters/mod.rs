use gasket::{
    messaging::{RecvPort, SendPort},
    runtime::Tether,
};
use serde::Deserialize;

use crate::framework::*;

pub mod deno;
pub mod dsl;
pub mod json;
pub mod legacy_v1;
pub mod match_pattern;
pub mod noop;
pub mod parse_cbor;
pub mod split_block;
pub mod wasm;

pub enum Bootstrapper {
    Noop(noop::Stage),
    SplitBlock(split_block::Stage),
    Dsl(dsl::Stage),
    Json(json::Stage),
    LegacyV1(legacy_v1::Stage),
    Wasm(wasm::Stage),
    Deno(deno::Stage),
    ParseCbor(parse_cbor::Stage),
    MatchPattern(match_pattern::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.input.connect(adapter),
            Bootstrapper::SplitBlock(p) => p.input.connect(adapter),
            Bootstrapper::Dsl(p) => p.input.connect(adapter),
            Bootstrapper::Json(p) => p.input.connect(adapter),
            Bootstrapper::LegacyV1(p) => p.input.connect(adapter),
            Bootstrapper::Wasm(p) => p.input.connect(adapter),
            Bootstrapper::Deno(p) => p.input.connect(adapter),
            Bootstrapper::ParseCbor(p) => p.input.connect(adapter),
            Bootstrapper::MatchPattern(p) => p.input.connect(adapter),
        }
    }

    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::Noop(p) => p.output.connect(adapter),
            Bootstrapper::SplitBlock(p) => p.output.connect(adapter),
            Bootstrapper::Dsl(p) => p.output.connect(adapter),
            Bootstrapper::Json(p) => p.output.connect(adapter),
            Bootstrapper::LegacyV1(p) => p.output.connect(adapter),
            Bootstrapper::Wasm(p) => p.output.connect(adapter),
            Bootstrapper::Deno(p) => p.output.connect(adapter),
            Bootstrapper::ParseCbor(p) => p.output.connect(adapter),
            Bootstrapper::MatchPattern(p) => p.output.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::SplitBlock(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Dsl(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Json(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::LegacyV1(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Wasm(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Deno(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::ParseCbor(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::MatchPattern(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Noop(noop::Config),
    SplitBlock(split_block::Config),
    Dsl(dsl::Config),
    Json(json::Config),
    LegacyV1(legacy_v1::Config),
    Wasm(wasm::Config),
    Deno(deno::Config),
    ParseCbor(parse_cbor::Config),
    MatchPattern(match_pattern::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
            Config::SplitBlock(c) => Ok(Bootstrapper::SplitBlock(c.bootstrapper(ctx)?)),
            Config::Dsl(c) => Ok(Bootstrapper::Dsl(c.bootstrapper(ctx)?)),
            Config::Json(c) => Ok(Bootstrapper::Json(c.bootstrapper(ctx)?)),
            Config::LegacyV1(c) => Ok(Bootstrapper::LegacyV1(c.bootstrapper(ctx)?)),
            Config::Wasm(c) => Ok(Bootstrapper::Wasm(c.bootstrapper(ctx)?)),
            Config::Deno(c) => Ok(Bootstrapper::Deno(c.bootstrapper(ctx)?)),
            Config::ParseCbor(c) => Ok(Bootstrapper::ParseCbor(c.bootstrapper(ctx)?)),
            Config::MatchPattern(c) => Ok(Bootstrapper::MatchPattern(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::LegacyV1(Default::default())
    }
}
