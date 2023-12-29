use gasket::{messaging::InputPort, runtime::Tether};
use pallas::network::miniprotocols::Point;
use serde::Deserialize;

use crate::framework::*;

pub mod file;
pub mod memory;

pub type MaxBreadcrums = usize;

pub enum Bootstrapper {
    Memory(memory::Stage),
    File(file::Stage),
}

impl Bootstrapper {
    pub fn borrow_track(&mut self) -> &mut InputPort<Point> {
        match self {
            Bootstrapper::Memory(x) => &mut x.track,
            Bootstrapper::File(x) => &mut x.track,
        }
    }

    pub fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Memory(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::File(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Memory(memory::Config),
    File(file::Config),
}

impl Config {
    pub fn initial_load(&self) -> Result<Breadcrumbs, Error> {
        match self {
            Config::Memory(x) => x.initial_load(),
            Config::File(x) => x.initial_load(),
        }
    }

    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Memory(c) => Ok(Bootstrapper::Memory(c.bootstrapper(ctx)?)),
            Config::File(c) => Ok(Bootstrapper::File(c.bootstrapper(ctx)?)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::Memory(memory::Config)
    }
}
