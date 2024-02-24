use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod dsl;
pub mod json;
pub mod legacy_v1;
pub mod match_pattern;
pub mod noop;
pub mod parse_cbor;
pub mod split_block;
pub mod wasm;

#[cfg(feature = "deno")]
pub mod deno;

pub enum Bootstrapper {
    Noop(noop::Stage),
    SplitBlock(split_block::Stage),
    Dsl(dsl::Stage),
    Json(json::Stage),
    LegacyV1(legacy_v1::Stage),
    Wasm(wasm::Stage),
    ParseCbor(parse_cbor::Stage),
    MatchPattern(match_pattern::Stage),

    #[cfg(feature = "deno")]
    Deno(deno::Stage),
}

impl Bootstrapper {
    pub fn borrow_input(&mut self) -> &mut FilterInputPort {
        match self {
            Bootstrapper::Noop(p) => &mut p.input,
            Bootstrapper::SplitBlock(p) => &mut p.input,
            Bootstrapper::Dsl(p) => &mut p.input,
            Bootstrapper::Json(p) => &mut p.input,
            Bootstrapper::LegacyV1(p) => &mut p.input,
            Bootstrapper::Wasm(p) => &mut p.input,
            Bootstrapper::ParseCbor(p) => &mut p.input,
            Bootstrapper::MatchPattern(p) => &mut p.input,

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(p) => &mut p.input,
        }
    }

    pub fn borrow_output(&mut self) -> &mut FilterOutputPort {
        match self {
            Bootstrapper::Noop(p) => &mut p.output,
            Bootstrapper::SplitBlock(p) => &mut p.output,
            Bootstrapper::Dsl(p) => &mut p.output,
            Bootstrapper::Json(p) => &mut p.output,
            Bootstrapper::LegacyV1(p) => &mut p.output,
            Bootstrapper::Wasm(p) => &mut p.output,
            Bootstrapper::ParseCbor(p) => &mut p.output,
            Bootstrapper::MatchPattern(p) => &mut p.output,

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(p) => &mut p.output,
        }
    }

    pub fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::SplitBlock(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Dsl(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Json(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::LegacyV1(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Wasm(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::ParseCbor(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::MatchPattern(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(x) => gasket::runtime::spawn_stage(x, policy),
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
    ParseCbor(parse_cbor::Config),
    MatchPattern(match_pattern::Config),

    #[cfg(feature = "deno")]
    Deno(deno::Config),
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
            Config::ParseCbor(c) => Ok(Bootstrapper::ParseCbor(c.bootstrapper(ctx)?)),
            Config::MatchPattern(c) => Ok(Bootstrapper::MatchPattern(c.bootstrapper(ctx)?)),

            #[cfg(feature = "deno")]
            Config::Deno(c) => Ok(Bootstrapper::Deno(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::LegacyV1(Default::default())
    }
}
