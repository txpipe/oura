use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

//pub mod assert;
//pub mod stdout;
pub mod noop;
pub mod terminal;

// #[cfg(feature = "logs")]
// pub mod logs;

// #[cfg(feature = "webhook")]
// pub mod webhook;

// #[cfg(feature = "kafkasink")]
// pub mod kafka;

// #[cfg(feature = "elasticsink")]
// pub mod elastic;

// #[cfg(feature = "aws")]
// pub mod aws_sqs;

// #[cfg(feature = "aws")]
// pub mod aws_lambda;

// #[cfg(feature = "aws")]
// pub mod aws_s3;

// #[cfg(feature = "redissink")]
// pub mod redis;

// #[cfg(feature = "gcp")]
// pub mod gcp_pubsub;

// #[cfg(feature = "gcp")]
// pub mod gcp_cloudfunction;

// #[cfg(feature = "rabbitmqsink")]
// pub mod rabbitmq;

pub enum Bootstrapper {
    Terminal(terminal::Bootstrapper),
    Noop(noop::Bootstrapper),
}

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: SinkInputAdapter) {
        match self {
            Bootstrapper::Terminal(p) => p.connect_input(adapter),
            Bootstrapper::Noop(p) => p.connect_input(adapter),
        }
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        match self {
            Bootstrapper::Terminal(x) => x.spawn(),
            Bootstrapper::Noop(x) => x.spawn(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Terminal(terminal::Config),
    Noop(noop::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Terminal(c) => Ok(Bootstrapper::Terminal(c.bootstrapper(ctx)?)),
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
        }
    }
}
