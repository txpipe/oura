use serde::Deserialize;

use crate::framework::*;

pub mod dsl;
pub mod noop;

pub enum Runtime {
    Noop(noop::Runtime),
    Dsl(dsl::Runtime),
    //Wasm,
}

pub enum Bootstrapper {
    Noop(noop::Bootstrapper),
    Dsl(dsl::Bootstrapper),
    //Wasm,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &mut FilterInputPort {
        match self {
            Bootstrapper::Noop(p) => p.borrow_input_port(),
            Bootstrapper::Dsl(p) => p.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &mut FilterOutputPort {
        match self {
            Bootstrapper::Noop(p) => p.borrow_output_port(),
            Bootstrapper::Dsl(p) => p.borrow_output_port(),
        }
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        match self {
            Bootstrapper::Noop(x) => Ok(Runtime::Noop(x.spawn()?)),
            Bootstrapper::Dsl(x) => Ok(Runtime::Dsl(x.spawn()?)),
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
