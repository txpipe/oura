use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

//#[cfg(target_family = "unix")]
//pub mod n2c;

pub mod n2n;

#[cfg(feature = "aws")]
pub mod s3;

pub enum Bootstrapper {
    N2N(n2n::Stage),
    N2C(),

    #[cfg(feature = "aws")]
    S3(s3::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::N2N(p) => p.connect_output(adapter),
            Bootstrapper::N2C() => todo!(),

            #[cfg(feature = "aws")]
            Bootstrapper::S3(p) => p.connect_output(adapter),
        }
    }

    fn connect_input(&mut self, _: InputAdapter) {
        panic!("attempted to use source stage as receiver");
    }

    fn spawn(self) -> Result<Vec<Tether>, Error> {
        match self {
            Bootstrapper::N2N(x) => x.spawn(),
            Bootstrapper::N2C() => todo!(),

            #[cfg(feature = "aws")]
            Bootstrapper::S3(x) => x.spawn(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    N2N(n2n::Config),

    #[cfg(target_family = "unix")]
    N2C,

    #[cfg(feature = "aws")]
    S3(s3::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::N2N(c) => Ok(Bootstrapper::N2N(c.bootstrapper(ctx)?)),
            Config::N2C => todo!(),

            #[cfg(feature = "aws")]
            Config::S3(c) => Ok(Bootstrapper::S3(c.bootstrapper(ctx)?)),
        }
    }
}
