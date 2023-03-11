use serde::Deserialize;

use crate::framework::*;

//#[cfg(target_family = "unix")]
//pub mod n2c;

pub mod n2n;

pub enum Runtime {
    N2N(n2n::Runtime),
    N2C(),
}

pub enum Bootstrapper {
    N2N(n2n::Bootstrapper),
    N2C(),
}

impl Bootstrapper {
    pub fn borrow_output_port(&mut self) -> &mut SourceOutputPort {
        match self {
            Bootstrapper::N2N(p) => p.borrow_output_port(),
            Bootstrapper::N2C() => todo!(),
        }
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        match self {
            Bootstrapper::N2N(x) => Ok(Runtime::N2N(x.spawn()?)),
            Bootstrapper::N2C() => todo!(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    N2N(n2n::Config),

    #[cfg(target_family = "unix")]
    N2C,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::N2N(c) => Ok(Bootstrapper::N2N(c.bootstrapper(ctx)?)),
            Config::N2C => todo!(),
        }
    }
}
