use serde::Deserialize;

use crate::framework::*;

pub mod json;
pub mod legacy_v1;
pub mod wasm;

pub enum Runtime {
    Json(json::Runtime),
    LegacyV1(legacy_v1::Runtime),
    Wasm(wasm::Runtime),
}

pub enum Bootstrapper {
    Json(json::Bootstrapper),
    LegacyV1(legacy_v1::Bootstrapper),
    Wasm(wasm::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &mut MapperInputPort {
        match self {
            Bootstrapper::Json(p) => p.borrow_input_port(),
            Bootstrapper::LegacyV1(p) => p.borrow_input_port(),
            Bootstrapper::Wasm(p) => p.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &mut MapperOutputPort {
        match self {
            Bootstrapper::Json(p) => p.borrow_output_port(),
            Bootstrapper::LegacyV1(p) => p.borrow_output_port(),
            Bootstrapper::Wasm(p) => p.borrow_output_port(),
        }
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        match self {
            Bootstrapper::Json(x) => Ok(Runtime::Json(x.spawn()?)),
            Bootstrapper::LegacyV1(x) => Ok(Runtime::LegacyV1(x.spawn()?)),
            Bootstrapper::Wasm(x) => Ok(Runtime::Wasm(x.spawn()?)),
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
