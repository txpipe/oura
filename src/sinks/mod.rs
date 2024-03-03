use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

mod assert;
mod common;
mod noop;
mod stdout;
mod terminal;

#[cfg(feature = "sink-file-rotate")]
mod file_rotate;

#[cfg(feature = "sink-webhook")]
mod webhook;

#[cfg(feature = "sink-rabbitmq")]
mod rabbitmq;

#[cfg(feature = "sink-kafka")]
mod kafka;

#[cfg(feature = "sink-aws-sqs")]
mod aws_sqs;

#[cfg(feature = "sink-aws-lambda")]
mod aws_lambda;

#[cfg(feature = "sink-aws-s3")]
mod aws_s3;

#[cfg(feature = "sink-gcp-pubsub")]
mod gcp_pubsub;

#[cfg(feature = "sink-gcp-cloudfunction")]
mod gcp_cloudfunction;

#[cfg(feature = "sink-redis")]
mod redis;

#[cfg(feature = "sink-elasticsearch")]
mod elasticsearch;

#[cfg(feature = "sql")]
mod sql_db;

pub enum Bootstrapper {
    Terminal(terminal::Stage),
    Stdout(stdout::Stage),
    Noop(noop::Stage),
    Assert(assert::Stage),

    #[cfg(feature = "sink-file-rotate")]
    FileRotate(file_rotate::Stage),

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

    #[cfg(feature = "sink-aws-s3")]
    AwsS3(aws_s3::Stage),

    #[cfg(feature = "sink-gcp-pubsub")]
    GcpPubSub(gcp_pubsub::Stage),

    #[cfg(feature = "sink-gcp-cloudfunction")]
    GcpCloudFunction(gcp_cloudfunction::Stage),

    #[cfg(feature = "sink-redis")]
    Redis(redis::Stage),

    #[cfg(feature = "sink-elasticsearch")]
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

            #[cfg(feature = "sink-file-rotate")]
            Bootstrapper::FileRotate(p) => &mut p.input,

            #[cfg(feature = "sink-webhook")]
            Bootstrapper::WebHook(p) => &mut p.input,

            #[cfg(feature = "sink-rabbitmq")]
            Bootstrapper::Rabbitmq(p) => &mut p.input,

            #[cfg(feature = "sink-kafka")]
            Bootstrapper::Kafka(p) => &mut p.input,

            #[cfg(feature = "sink-aws-sqs")]
            Bootstrapper::AwsSqs(p) => &mut p.input,

            #[cfg(feature = "sink-aws-lambda")]
            Bootstrapper::AwsLambda(p) => &mut p.input,

            #[cfg(feature = "sink-aws-s3")]
            Bootstrapper::AwsS3(p) => &mut p.input,

            #[cfg(feature = "sink-gcp-pubsub")]
            Bootstrapper::GcpPubSub(p) => &mut p.input,

            #[cfg(feature = "sink-gcp-cloudfunction")]
            Bootstrapper::GcpCloudFunction(p) => &mut p.input,

            #[cfg(feature = "sink-redis")]
            Bootstrapper::Redis(p) => &mut p.input,

            #[cfg(feature = "sink-elasticsearch")]
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

            #[cfg(feature = "sink-file-rotate")]
            Bootstrapper::FileRotate(p) => &mut p.cursor,

            #[cfg(feature = "sink-webhook")]
            Bootstrapper::WebHook(p) => &mut p.cursor,

            #[cfg(feature = "sink-rabbitmq")]
            Bootstrapper::Rabbitmq(p) => &mut p.cursor,

            #[cfg(feature = "sink-kafka")]
            Bootstrapper::Kafka(p) => &mut p.cursor,

            #[cfg(feature = "sink-aws-sqs")]
            Bootstrapper::AwsSqs(p) => &mut p.cursor,

            #[cfg(feature = "sink-aws-lambda")]
            Bootstrapper::AwsLambda(p) => &mut p.cursor,

            #[cfg(feature = "sink-aws-s3")]
            Bootstrapper::AwsS3(p) => &mut p.cursor,

            #[cfg(feature = "sink-gcp-pubsub")]
            Bootstrapper::GcpPubSub(p) => &mut p.cursor,

            #[cfg(feature = "sink-gcp-cloudfunction")]
            Bootstrapper::GcpCloudFunction(p) => &mut p.cursor,

            #[cfg(feature = "sink-redis")]
            Bootstrapper::Redis(p) => &mut p.cursor,

            #[cfg(feature = "sink-elasticsearch")]
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

            #[cfg(feature = "sink-file-rotate")]
            Bootstrapper::FileRotate(x) => gasket::runtime::spawn_stage(x, policy),

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

            #[cfg(feature = "sink-aws-s3")]
            Bootstrapper::AwsS3(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-gcp-pubsub")]
            Bootstrapper::GcpPubSub(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-gcp-cloudfunction")]
            Bootstrapper::GcpCloudFunction(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-redis")]
            Bootstrapper::Redis(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "sink-elasticsearch")]
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

    #[cfg(feature = "sink-file-rotate")]
    FileRotate(file_rotate::Config),

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

    #[cfg(feature = "sink-aws-s3")]
    AwsS3(aws_s3::Config),

    #[cfg(feature = "sink-gcp-pubsub")]
    GcpPubSub(gcp_pubsub::Config),

    #[cfg(feature = "sink-gcp-cloudfunction")]
    GcpCloudFunction(gcp_cloudfunction::Config),

    #[cfg(feature = "sink-redis")]
    Redis(redis::Config),

    #[cfg(feature = "sink-elasticsearch")]
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

            #[cfg(feature = "sink-file-rotate")]
            Config::FileRotate(c) => Ok(Bootstrapper::FileRotate(c.bootstrapper(ctx)?)),

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

            #[cfg(feature = "sink-aws-s3")]
            Config::AwsS3(c) => Ok(Bootstrapper::AwsS3(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-gcp-pubsub")]
            Config::GcpPubSub(c) => Ok(Bootstrapper::GcpPubSub(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-gcp-cloudfunction")]
            Config::GcpCloudFunction(c) => Ok(Bootstrapper::GcpCloudFunction(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-redis")]
            Config::Redis(c) => Ok(Bootstrapper::Redis(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sink-elasticsearch")]
            Config::ElasticSearch(c) => Ok(Bootstrapper::ElasticSearch(c.bootstrapper(ctx)?)),

            #[cfg(feature = "sql")]
            Config::SqlDb(c) => Ok(Bootstrapper::SqlDb(c.bootstrapper(ctx)?)),
        }
    }
}
