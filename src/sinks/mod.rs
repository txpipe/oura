use gasket::{messaging::RecvPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::*;

//pub mod assert;
pub mod filerotate;
pub mod noop;
pub mod stdout;
pub mod terminal;

#[cfg(feature = "sink-webhook")]
pub mod webhook;

#[cfg(feature = "sink-rabbitmq")]
mod rabbitmq;

#[cfg(feature = "sink-kafka")]
mod kafka;

#[cfg(feature = "sink-aws-sqs")]
mod aws_sqs;

#[cfg(feature = "sink-aws-lambda")]
mod aws_lambda;

#[cfg(feature = "sink-redis")]
mod redis;

#[cfg(feature = "sink-elasticsearch")]
mod elasticsearch;

// #[cfg(feature = "aws")]
// pub mod aws_s3;

// #[cfg(feature = "gcp")]
// pub mod gcp_pubsub;

// #[cfg(feature = "gcp")]
// pub mod gcp_cloudfunction;

pub enum Bootstrapper {
    Terminal(terminal::Stage),
    FileRotate(filerotate::Stage),
    Stdout(stdout::Stage),
    Noop(noop::Stage),

    #[cfg(feature = "sink-webhook")]
    WebHook(webhook::Stage),

    #[cfg(feature = "sink-rabbitmq")]
    Rabbitmq(rabbitmq::Stage),

    #[cfg(feature = "sink-kafka")]
    Kafka(kafka::Stage),

    #[cfg(feature = "sink-aws-sqs")]
    AwsSqs(aws_sqs::Stage),

    #[cfg(feature = "sink-aws-lambda")]
    AwsLambda(aws_lambda::Stage),

    #[cfg(feature = "sink-redis")]
    Redis(redis::Stage),

    #[cfg(feature = "sink-elasticsearch")]
    ElasticSearch(elasticsearch::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, _: OutputAdapter) {
        panic!("attempted to use sink stage as sender");
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Terminal(p) => p.input.connect(adapter),
            Bootstrapper::FileRotate(p) => p.input.connect(adapter),
            Bootstrapper::Stdout(p) => p.input.connect(adapter),
            Bootstrapper::Noop(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-webhook")]
            Bootstrapper::WebHook(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-rabbitmq")]
            Bootstrapper::Rabbitmq(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-kafka")]
            Bootstrapper::Kafka(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-aws-sqs")]
            Bootstrapper::AwsSqs(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-aws-lambda")]
            Bootstrapper::AwsLambda(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-redis")]
            Bootstrapper::Redis(p) => p.input.connect(adapter),

            #[cfg(feature = "sink-elasticsearch")]
            Bootstrapper::ElasticSearch(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Terminal(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::FileRotate(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Stdout(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-webhook")]
            Bootstrapper::WebHook(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-rabbitmq")]
            Bootstrapper::Rabbitmq(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-kafka")]
            Bootstrapper::Kafka(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-aws-sqs")]
            Bootstrapper::AwsSqs(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-aws-lambda")]
            Bootstrapper::AwsLambda(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-redis")]
            Bootstrapper::Redis(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-elasticsearch")]
            Bootstrapper::ElasticSearch(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Terminal(terminal::Config),
    FileRotate(filerotate::Config),
    Stdout(stdout::Config),
    Noop(noop::Config),

    #[cfg(feature = "sink-webhook")]
    WebHook(webhook::Config),

    #[cfg(feature = "sink-rabbitmq")]
    Rabbitmq(rabbitmq::Config),

    #[cfg(feature = "sink-kafka")]
    Kafka(kafka::Config),

    #[cfg(feature = "sink-aws-sqs")]
    AwsSqs(aws_sqs::Config),

    #[cfg(feature = "sink-aws-lambda")]
    AwsLambda(aws_lambda::Config),

    #[cfg(feature = "sink-redis")]
    Redis(redis::Config),

    #[cfg(feature = "sink-elasticsearch")]
    ElasticSearch(elasticsearch::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Terminal(c) => Ok(Bootstrapper::Terminal(c.bootstrapper(ctx)?)),
            Config::FileRotate(c) => Ok(Bootstrapper::FileRotate(c.bootstrapper(ctx)?)),
            Config::Stdout(c) => Ok(Bootstrapper::Stdout(c.bootstrapper(ctx)?)),
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-webhook")]
            Config::WebHook(c) => Ok(Bootstrapper::WebHook(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-rabbitmq")]
            Config::Rabbitmq(c) => Ok(Bootstrapper::Rabbitmq(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-kafka")]
            Config::Kafka(c) => Ok(Bootstrapper::Kafka(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-aws-sqs")]
            Config::AwsSqs(c) => Ok(Bootstrapper::AwsSqs(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-aws-lambda")]
            Config::AwsLambda(c) => Ok(Bootstrapper::AwsLambda(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-redis")]
            Config::Redis(c) => Ok(Bootstrapper::Redis(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-elasticsearch")]
            Config::ElasticSearch(c) => Ok(Bootstrapper::ElasticSearch(c.bootstrapper(ctx)?)),
        }
    }
}
