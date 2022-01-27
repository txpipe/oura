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
    utils::{ChainWellKnownInfo, Utils, WithUtils},
    Error,
};

use oura::filters::noop::Config as NoopFilterConfig;
use oura::filters::selection::Config as SelectionConfig;
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

    #[cfg(feature = "logs")]
    Logs(WriterConfig),

    #[cfg(feature = "webhook")]
    Webhook(WebhookConfig),

    #[cfg(feature = "kafkasink")]
    Kafka(KafkaConfig),

    #[cfg(feature = "elasticsink")]
    Elastic(ElasticConfig),
}

impl SinkProvider for Sink {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        match self {
            Sink::Terminal(c) => c.bootstrap(input),
            Sink::Stdout(c) => c.bootstrap(input),

            #[cfg(feature = "logs")]
            Sink::Logs(c) => c.bootstrap(input),

            #[cfg(feature = "webhook")]
            Sink::Webhook(c) => c.bootstrap(input),

            #[cfg(feature = "kafkasink")]
            Sink::Kafka(c) => c.bootstrap(input),

            #[cfg(feature = "elasticsink")]
            Sink::Elastic(c) => c.bootstrap(input),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigRoot {
    source: Source,

    #[serde(default)]
    filters: Vec<Filter>,

    sink: Sink,

    chain: Option<ChainWellKnownInfo>,
}

impl ConfigRoot {
    pub fn new(explicit_file: Option<String>) -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // our base config will always be in /etc/oura
        s.merge(File::with_name("/etc/oura/daemon.toml").required(false))?;

        // but we can override it by having a file in the working dir
        s.merge(File::with_name("oura.toml").required(false))?;

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file {
            s.merge(File::with_name(&explicit).required(true))?;
        }

        // finally, we use env vars to make some last-step overrides
        s.merge(Environment::with_prefix("OURA").separator("_"))?;

        s.try_into()
    }
}

/// Sets up the whole pipeline from configuration
fn bootstrap(config: ConfigRoot) -> Result<Vec<JoinHandle<()>>, Error> {
    let well_known = config.chain.unwrap_or_default();

    let utils = Arc::new(Utils::new(well_known));

    let mut threads = Vec::with_capacity(10);

    let (source_handle, source_rx) = bootstrap_source(config.source, utils)?;
    threads.push(source_handle);

    let mut last_rx = source_rx;

    for filter in config.filters.iter() {
        let (filter_handle, filter_rx) = filter.bootstrap(last_rx)?;
        threads.push(filter_handle);
        last_rx = filter_rx;
    }

    let sink_handle = config.sink.bootstrap(last_rx)?;
    threads.push(sink_handle);

    Ok(threads)
}

pub fn run(args: &ArgMatches) -> Result<(), Error> {
    env_logger::init();

    let explicit_config = match args.is_present("config") {
        true => Some(args.value_of_t("config")?),
        false => None,
    };

    let root = ConfigRoot::new(explicit_config)?;

    debug!("daemon starting with this config: {:?}", root);

    let threads = bootstrap(root)?;

    // TODO: refactor into new loop that monitors thread health
    for handle in threads {
        handle.join().expect("error in pipeline thread");
    }

    Ok(())
}
