use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod into_json;
pub mod legacy_v1;
pub mod noop;
pub mod parse_cbor;
pub mod select;
pub mod split_block;
pub mod split_tx;

#[cfg(feature = "wasm")]
pub mod wasm_plugin;

pub enum Bootstrapper {
    Noop(noop::Stage),
    SplitBlock(split_block::Stage),
    SplitTx(split_tx::Stage),
    IntoJson(into_json::Stage),
    LegacyV1(legacy_v1::Stage),
    ParseCbor(parse_cbor::Stage),
    Select(select::Stage),

    #[cfg(feature = "wasm")]
    WasmPlugin(wasm_plugin::Stage),
}

impl Bootstrapper {
    pub fn borrow_input(&mut self) -> &mut FilterInputPort {
        match self {
            Bootstrapper::Noop(p) => &mut p.input,
            Bootstrapper::SplitBlock(p) => &mut p.input,
            Bootstrapper::SplitTx(p) => &mut p.input,
            Bootstrapper::IntoJson(p) => &mut p.input,
            Bootstrapper::LegacyV1(p) => &mut p.input,
            Bootstrapper::ParseCbor(p) => &mut p.input,
            Bootstrapper::Select(p) => &mut p.input,

            #[cfg(feature = "wasm")]
            Bootstrapper::WasmPlugin(p) => &mut p.input,
        }
    }

    pub fn borrow_output(&mut self) -> &mut FilterOutputPort {
        match self {
            Bootstrapper::Noop(p) => &mut p.output,
            Bootstrapper::SplitBlock(p) => &mut p.output,
            Bootstrapper::SplitTx(p) => &mut p.output,
            Bootstrapper::IntoJson(p) => &mut p.output,
            Bootstrapper::LegacyV1(p) => &mut p.output,
            Bootstrapper::ParseCbor(p) => &mut p.output,
            Bootstrapper::Select(p) => &mut p.output,

            #[cfg(feature = "wasm")]
            Bootstrapper::WasmPlugin(p) => &mut p.output,
        }
    }

    pub fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::SplitBlock(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::SplitTx(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::IntoJson(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::LegacyV1(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::ParseCbor(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Select(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "wasm")]
            Bootstrapper::WasmPlugin(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Noop(noop::Config),
    SplitBlock(split_block::Config),
    SplitTx(split_tx::Config),
    IntoJson(into_json::Config),
    LegacyV1(legacy_v1::Config),
    ParseCbor(parse_cbor::Config),
    Select(select::Config),

    #[cfg(feature = "wasm")]
    WasmPlugin(wasm_plugin::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
            Config::SplitBlock(c) => Ok(Bootstrapper::SplitBlock(c.bootstrapper(ctx)?)),
            Config::SplitTx(c) => Ok(Bootstrapper::SplitTx(c.bootstrapper(ctx)?)),
            Config::IntoJson(c) => Ok(Bootstrapper::IntoJson(c.bootstrapper(ctx)?)),
            Config::LegacyV1(c) => Ok(Bootstrapper::LegacyV1(c.bootstrapper(ctx)?)),
            Config::ParseCbor(c) => Ok(Bootstrapper::ParseCbor(c.bootstrapper(ctx)?)),
            Config::Select(c) => Ok(Bootstrapper::Select(c.bootstrapper(ctx)?)),

            #[cfg(feature = "wasm")]
            Config::WasmPlugin(c) => Ok(Bootstrapper::WasmPlugin(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::LegacyV1(Default::default())
    }
}
