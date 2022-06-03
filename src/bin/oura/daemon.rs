use std::{sync::Arc, thread::JoinHandle};

use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use log::debug;
use serde::Deserialize;

use oura::{
    pipelining::{
        BootstrapResult, FilterProvider, PartialBootstrapResult, SinkProvider, SourceProvider,
        StageReceiver,
    },
    sources::{MagicArg, PointArg},
    utils::{cursor, metrics, ChainWellKnownInfo, Utils, WithUtils},
    Error,
};

use oura::filters::noop::Config as NoopFilterConfig;
use oura::filters::selection::Config as SelectionConfig;
use oura::sinks::assert::Config as AssertConfig;
use oura::sinks::stdout::Config as StdoutConfig;
use oura::sinks::terminal::Config as TerminalConfig;
use oura::sources::n2c::Config as N2CConfig;
use oura::sources::n2n::Config as N2NConfig;

#[cfg(feature = "logs")]
use oura::sinks::logs::Config as WriterConfig;

#[cfg(feature = "webhook")]
use oura::sinks::webhook::Config as WebhookConfig;

#[cfg(feature = "kafkasink")]
use oura::sinks::kafka::Config as KafkaConfig;

#[cfg(feature = "elasticsink")]
use oura::sinks::elastic::Config as ElasticConfig;

#[cfg(feature = "aws")]
use oura::sinks::aws_sqs::Config as AwsSqsConfig;

#[cfg(feature = "aws")]
use oura::sinks::aws_lambda::Config as AwsLambdaConfig;

#[cfg(feature = "aws")]
use oura::sinks::aws_s3::Config as AwsS3Config;

#[cfg(feature = "redissink")]
use oura::sinks::redis::Config as RedisConfig;

#[cfg(feature = "gcp")]
use oura::sinks::gcp_pubsub::Config as GcpPubSubConfig;

#[cfg(feature = "fingerprint")]
use oura::filters::fingerprint::Config as FingerprintConfig;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Source {
    N2C(N2CConfig),
    N2N(N2NConfig),
}

fn bootstrap_source(config: Source, utils: Arc<Utils>) -> PartialBootstrapResult {
    match config {
        Source::N2C(config) => WithUtils::new(config, utils).bootstrap(),
        Source::N2N(config) => WithUtils::new(config, utils).bootstrap(),
    }
}

fn infer_magic_from_source(config: &Source) -> Option<MagicArg> {
    match config {
        Source::N2C(config) => config.magic.clone(),
        Source::N2N(config) => config.magic.clone(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Filter {
    Noop(NoopFilterConfig),
    Selection(SelectionConfig),

    #[cfg(feature = "fingerprint")]
    Fingerprint(FingerprintConfig),
}

impl FilterProvider for Filter {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult {
        match self {
            Filter::Noop(c) => c.bootstrap(input),
            Filter::Selection(c) => c.bootstrap(input),

            #[cfg(feature = "fingerprint")]
            Filter::Fingerprint(c) => c.bootstrap(input),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Sink {
    Terminal(TerminalConfig),
    Stdout(StdoutConfig),
    Assert(AssertConfig),

    #[cfg(feature = "logs")]
    Logs(WriterConfig),

    #[cfg(feature = "webhook")]
    Webhook(WebhookConfig),

    #[cfg(feature = "kafkasink")]
    Kafka(KafkaConfig),

    #[cfg(feature = "elasticsink")]
    Elastic(ElasticConfig),

    #[cfg(feature = "aws")]
    AwsSqs(AwsSqsConfig),

    #[cfg(feature = "aws")]
    AwsLambda(AwsLambdaConfig),

    #[cfg(feature = "aws")]
    AwsS3(AwsS3Config),

    #[cfg(feature = "redissink")]
    Redis(RedisConfig),

    #[cfg(feature = "gcp")]
    GcpPubSub(GcpPubSubConfig),
}

fn bootstrap_sink(config: Sink, input: StageReceiver, utils: Arc<Utils>) -> BootstrapResult {
    match config {
        Sink::Terminal(c) => WithUtils::new(c, utils).bootstrap(input),
        Sink::Stdout(c) => WithUtils::new(c, utils).bootstrap(input),
        Sink::Assert(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "logs")]
        Sink::Logs(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "webhook")]
        Sink::Webhook(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "kafkasink")]
        Sink::Kafka(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "elasticsink")]
        Sink::Elastic(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "aws")]
        Sink::AwsSqs(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "aws")]
        Sink::AwsLambda(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "aws")]
        Sink::AwsS3(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "redissink")]
        Sink::Redis(c) => WithUtils::new(c, utils).bootstrap(input),

        #[cfg(feature = "gcp")]
        Sink::GcpPubSub(c) => WithUtils::new(c, utils).bootstrap(input),
    }
}

#[derive(Debug, Deserialize)]
struct ConfigRoot {
    source: Source,

    #[serde(default)]
    filters: Vec<Filter>,

    sink: Sink,

    chain: Option<ChainWellKnownInfo>,

    cursor: Option<cursor::Config>,

    metrics: Option<metrics::Config>,
}

impl ConfigRoot {
    pub fn new(explicit_file: Option<String>) -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // our base config will always be in /etc/oura
        s = s.add_source(File::with_name("/etc/oura/daemon.toml").required(false));

        // but we can override it by having a file in the working dir
        s = s.add_source(File::with_name("oura.toml").required(false));

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file {
            s = s.add_source(File::with_name(&explicit).required(true));
        }

        // finally, we use env vars to make some last-step overrides
        s = s.add_source(Environment::with_prefix("OURA").separator("_"));

        s.build()?.try_deserialize()
    }
}

fn define_chain_info(
    explicit: Option<ChainWellKnownInfo>,
    magic: &MagicArg,
) -> Result<ChainWellKnownInfo, Error> {
    match explicit {
        Some(x) => Ok(x),
        None => ChainWellKnownInfo::try_from_magic(**magic),
    }
}

fn define_cursor(
    explicit: Option<PointArg>,
    config: Option<cursor::Config>,
) -> Option<cursor::Config> {
    match (explicit, config) {
        (Some(x), _) => Some(cursor::Config::Memory(x)),
        (_, x) => x,
    }
}

fn bootstrap_utils(
    chain: ChainWellKnownInfo,
    cursor: Option<cursor::Config>,
    metrics: Option<metrics::Config>,
) -> Utils {
    let mut utils = Utils::new(chain);

    if let Some(cursor) = cursor {
        utils = utils.with_cursor(cursor);
    }

    if let Some(metrics) = metrics {
        utils = utils.with_metrics(metrics);
    }

    utils
}

/// Sets up the whole pipeline from configuration
fn bootstrap(
    config: ConfigRoot,
    explicit_cursor: Option<PointArg>,
) -> Result<Vec<JoinHandle<()>>, Error> {
    let ConfigRoot {
        source,
        filters,
        sink,
        chain,
        cursor,
        metrics,
    } = config;

    let magic = infer_magic_from_source(&source).unwrap_or_default();

    let chain = define_chain_info(chain, &magic)?;

    let cursor = define_cursor(explicit_cursor, cursor);

    let utils = Arc::new(bootstrap_utils(chain, cursor, metrics));

    let mut threads = Vec::with_capacity(10);

    let (source_handle, source_rx) = bootstrap_source(source, utils.clone())?;
    threads.push(source_handle);

    let mut last_rx = source_rx;

    for filter in filters.iter() {
        let (filter_handle, filter_rx) = filter.bootstrap(last_rx)?;
        threads.push(filter_handle);
        last_rx = filter_rx;
    }

    let sink_handle = bootstrap_sink(sink, last_rx, utils)?;
    threads.push(sink_handle);

    Ok(threads)
}

pub fn run(args: &ArgMatches) -> Result<(), Error> {
    env_logger::init();

    let explicit_config = match args.is_present("config") {
        true => Some(args.value_of_t("config")?),
        false => None,
    };

    let explicit_cursor = match args.is_present("cursor") {
        true => Some(args.value_of_t("cursor")?),
        false => None,
    };

    let root = ConfigRoot::new(explicit_config)?;

    debug!("daemon starting with this config: {:?}", root);

    let threads = bootstrap(root, explicit_cursor)?;

    // TODO: refactor into new loop that monitors thread health
    for handle in threads {
        handle.join().expect("error in pipeline thread");
    }

    Ok(())
}

/// Creates the clap definition for this sub-command
pub(crate) fn command_definition<'a>() -> clap::Command<'a> {
    clap::Command::new("daemon")
        .arg(
            clap::Arg::new("config")
                .long("config")
                .takes_value(true)
                .help("config file to load by the daemon"),
        )
        .arg(
            clap::Arg::new("cursor")
                .long("cursor")
                .takes_value(true)
                .help(
                    "initial chain cursor, overrides configuration file, expects format `slot,hex-hash`",
                ),
        )
}
