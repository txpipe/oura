use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

pub mod assert;
pub mod common;
pub mod file_rotate;
pub mod noop;
pub mod stdout;
pub mod terminal;
pub mod webhook;

#[cfg(feature = "rabbitmq")]
mod rabbitmq;

#[cfg(feature = "kafka")]
mod kafka;

#[cfg(feature = "aws")]
mod aws_sqs;

#[cfg(feature = "aws")]
mod aws_lambda;

#[cfg(feature = "aws")]
mod aws_s3;

#[cfg(feature = "gcp")]
mod gcp_pubsub;

#[cfg(feature = "gcp")]
mod gcp_cloudfunction;

#[cfg(feature = "redis")]
mod redis;

#[cfg(feature = "elasticsearch")]
mod elasticsearch;

#[cfg(feature = "sql")]
mod sql_db;

pub enum Bootstrapper {
    Terminal(terminal::Stage),
    Stdout(stdout::Stage),
    Noop(noop::Stage),
    Assert(assert::Stage),
    FileRotate(file_rotate::Stage),
    WebHook(webhook::Stage),

    #[cfg(feature = "rabbitmq")]
    Rabbitmq(rabbitmq::Stage),

    #[cfg(feature = "kafka")]
    Kafka(kafka::Stage),

    #[cfg(feature = "aws")]
    AwsSqs(aws_sqs::Stage),

    #[cfg(feature = "aws")]
    AwsLambda(aws_lambda::Stage),

    #[cfg(feature = "aws")]
    AwsS3(aws_s3::Stage),

    #[cfg(feature = "gcp")]
    GcpPubSub(gcp_pubsub::Stage),

    #[cfg(feature = "gcp")]
    GcpCloudFunction(gcp_cloudfunction::Stage),

    #[cfg(feature = "redis")]
    Redis(redis::Stage),

    #[cfg(feature = "elasticsearch")]
    ElasticSearch(elasticsearch::Stage),

    #[cfg(feature = "sql")]
    SqlDb(sql_db::Stage),
}

impl Bootstrapper {
    pub fn borrow_input(&mut self) -> &mut SinkInputPort {
        match self {
            Bootstrapper::Terminal(p) => &mut p.input,
            Bootstrapper::Stdout(p) => &mut p.input,
            Bootstrapper::Noop(p) => &mut p.input,
            Bootstrapper::Assert(p) => &mut p.input,
            Bootstrapper::FileRotate(p) => &mut p.input,
            Bootstrapper::WebHook(p) => &mut p.input,

            #[cfg(feature = "rabbitmq")]
            Bootstrapper::Rabbitmq(p) => &mut p.input,

            #[cfg(feature = "kafka")]
            Bootstrapper::Kafka(p) => &mut p.input,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsSqs(p) => &mut p.input,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsLambda(p) => &mut p.input,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsS3(p) => &mut p.input,

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpPubSub(p) => &mut p.input,

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpCloudFunction(p) => &mut p.input,

            #[cfg(feature = "redis")]
            Bootstrapper::Redis(p) => &mut p.input,

            #[cfg(feature = "elasticsearch")]
            Bootstrapper::ElasticSearch(p) => &mut p.input,

            #[cfg(feature = "sql")]
            Bootstrapper::SqlDb(p) => &mut p.input,
        }
    }

    pub fn borrow_cursor(&mut self) -> &mut SinkCursorPort {
        match self {
            Bootstrapper::Terminal(p) => &mut p.cursor,
            Bootstrapper::Stdout(p) => &mut p.cursor,
            Bootstrapper::Noop(p) => &mut p.cursor,
            Bootstrapper::Assert(p) => &mut p.cursor,
            Bootstrapper::FileRotate(p) => &mut p.cursor,
            Bootstrapper::WebHook(p) => &mut p.cursor,

            #[cfg(feature = "rabbitmq")]
            Bootstrapper::Rabbitmq(p) => &mut p.cursor,

            #[cfg(feature = "kafka")]
            Bootstrapper::Kafka(p) => &mut p.cursor,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsSqs(p) => &mut p.cursor,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsLambda(p) => &mut p.cursor,

            #[cfg(feature = "aws")]
            Bootstrapper::AwsS3(p) => &mut p.cursor,

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpPubSub(p) => &mut p.cursor,

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpCloudFunction(p) => &mut p.cursor,

            #[cfg(feature = "redis")]
            Bootstrapper::Redis(p) => &mut p.cursor,

            #[cfg(feature = "elasticsearch")]
            Bootstrapper::ElasticSearch(p) => &mut p.cursor,

            #[cfg(feature = "sql")]
            Bootstrapper::SqlDb(p) => &mut p.cursor,
        }
    }

    pub fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Terminal(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Stdout(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Noop(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Assert(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::FileRotate(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::WebHook(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "rabbitmq")]
            Bootstrapper::Rabbitmq(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "kafka")]
            Bootstrapper::Kafka(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "aws")]
            Bootstrapper::AwsSqs(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "aws")]
            Bootstrapper::AwsLambda(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "aws")]
            Bootstrapper::AwsS3(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpPubSub(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "gcp")]
            Bootstrapper::GcpCloudFunction(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "redis")]
            Bootstrapper::Redis(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "elasticsearch")]
            Bootstrapper::ElasticSearch(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sql")]
            Bootstrapper::SqlDb(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Terminal(terminal::Config),
    Stdout(stdout::Config),
    Noop(noop::Config),
    Assert(assert::Config),
    FileRotate(file_rotate::Config),
    WebHook(webhook::Config),

    #[cfg(feature = "rabbitmq")]
    Rabbitmq(rabbitmq::Config),

    #[cfg(feature = "kafka")]
    Kafka(kafka::Config),

    #[cfg(feature = "aws")]
    AwsSqs(aws_sqs::Config),

    #[cfg(feature = "aws")]
    AwsLambda(aws_lambda::Config),

    #[cfg(feature = "aws")]
    AwsS3(aws_s3::Config),

    #[cfg(feature = "gcp")]
    GcpPubSub(gcp_pubsub::Config),

    #[cfg(feature = "gcp")]
    GcpCloudFunction(gcp_cloudfunction::Config),

    #[cfg(feature = "redis")]
    Redis(redis::Config),

    #[cfg(feature = "elasticsearch")]
    ElasticSearch(elasticsearch::Config),

    #[cfg(feature = "sql")]
    SqlDb(sql_db::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Terminal(c) => Ok(Bootstrapper::Terminal(c.bootstrapper(ctx)?)),
            Config::Stdout(c) => Ok(Bootstrapper::Stdout(c.bootstrapper(ctx)?)),
            Config::Noop(c) => Ok(Bootstrapper::Noop(c.bootstrapper(ctx)?)),
            Config::Assert(c) => Ok(Bootstrapper::Assert(c.bootstrapper(ctx)?)),
            Config::FileRotate(c) => Ok(Bootstrapper::FileRotate(c.bootstrapper(ctx)?)),
            Config::WebHook(c) => Ok(Bootstrapper::WebHook(c.bootstrapper(ctx)?)),

            #[cfg(feature = "rabbitmq")]
            Config::Rabbitmq(c) => Ok(Bootstrapper::Rabbitmq(c.bootstrapper(ctx)?)),

            #[cfg(feature = "kafka")]
            Config::Kafka(c) => Ok(Bootstrapper::Kafka(c.bootstrapper(ctx)?)),

            #[cfg(feature = "aws")]
            Config::AwsSqs(c) => Ok(Bootstrapper::AwsSqs(c.bootstrapper(ctx)?)),

            #[cfg(feature = "aws")]
            Config::AwsLambda(c) => Ok(Bootstrapper::AwsLambda(c.bootstrapper(ctx)?)),

            #[cfg(feature = "aws")]
            Config::AwsS3(c) => Ok(Bootstrapper::AwsS3(c.bootstrapper(ctx)?)),

            #[cfg(feature = "gcp")]
            Config::GcpPubSub(c) => Ok(Bootstrapper::GcpPubSub(c.bootstrapper(ctx)?)),

            #[cfg(feature = "gcp")]
            Config::GcpCloudFunction(c) => Ok(Bootstrapper::GcpCloudFunction(c.bootstrapper(ctx)?)),

            #[cfg(feature = "redis")]
            Config::Redis(c) => Ok(Bootstrapper::Redis(c.bootstrapper(ctx)?)),

            #[cfg(feature = "elasticsearch")]
            Config::ElasticSearch(c) => Ok(Bootstrapper::ElasticSearch(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sql")]
            Config::SqlDb(c) => Ok(Bootstrapper::SqlDb(c.bootstrapper(ctx)?)),
        }
    }
}
