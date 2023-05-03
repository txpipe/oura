use gasket::{messaging::RecvPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::*;

//pub mod assert;
//pub mod stdout;
pub mod filerotate;
pub mod noop;
pub mod terminal;
pub mod webhook;

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
    Terminal(terminal::Stage),
    FileRotate(filerotate::Stage),
    WebHook(webhook::Stage),
    Noop(noop::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, _: OutputAdapter) {
        panic!("attempted to use sink stage as sender");
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Terminal(p) => p.input.connect(adapter),
            Bootstrapper::FileRotate(p) => p.input.connect(adapter),
            Bootstrapper::WebHook(p) => p.input.connect(adapter),
            Bootstrapper::Noop(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Terminal(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::FileRotate(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::WebHook(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Terminal(terminal::Config),
    FileRotate(filerotate::Config),
    WebHook(webhook::Config),
    Noop(noop::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Terminal(c) => Ok(Bootstrapper::Terminal(c.bootstrapper(ctx)?)),
            Config::FileRotate(c) => Ok(Bootstrapper::FileRotate(c.bootstrapper(ctx)?)),
            Config::WebHook(c) => Ok(Bootstrapper::WebHook(c.bootstrapper(ctx)?)),
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
        }
    }
}
