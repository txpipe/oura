use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use clap::{value_t, ArgMatches};
use config::{Config, ConfigError, Environment, File};
use log::debug;
use oura::framework::{
    BootstrapResult, Event, FilterConfig, PartialBootstrapResult, SinkConfig, SourceConfig,
};
use oura::sinks::terminal::Config as TerminalConfig;
use oura::sources::n2c::Config as N2CConfig;
use oura::sources::n2n::Config as N2NConfig;
use serde_derive::Deserialize;

#[cfg(feature = "kafkasink")]
use oura::sinks::kafka::Config as KafkaConfig;

#[cfg(feature = "elasticsink")]
use oura::sinks::elastic::Config as ElasticConfig;

use oura::filters::noop::Config as NoopFilterConfig;

use crate::Error;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Source {
    N2C(N2CConfig),
    N2N(N2NConfig),
}

impl SourceConfig for Source {
    fn bootstrap(&self) -> PartialBootstrapResult {
        match self {
            Source::N2C(c) => c.bootstrap(),
            Source::N2N(c) => c.bootstrap(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Filter {
    Noop(NoopFilterConfig),
}

impl FilterConfig for Filter {
    fn bootstrap(&self, input: Receiver<Event>) -> PartialBootstrapResult {
        match self {
            Filter::Noop(c) => c.bootstrap(input),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Sink {
    Terminal(TerminalConfig),

    #[cfg(feature = "kafkasink")]
    Kafka(KafkaConfig),

    #[cfg(feature = "elasticsink")]
    Elastic(ElasticConfig),
}

impl SinkConfig for Sink {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult {
        match self {
            Sink::Terminal(c) => c.bootstrap(input),

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
fn bootstrap(config: &ConfigRoot) -> Result<Vec<JoinHandle<()>>, Error> {
    let mut threads = Vec::with_capacity(10);

    let (source_handle, source_rx) = config.source.bootstrap()?;
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
        true => Some(value_t!(args, "config", String)?),
        false => None,
    };

    let root = ConfigRoot::new(explicit_config)?;

    debug!("daemon starting with this config: {:?}", root);

    let threads = bootstrap(&root)?;

    // TODO: refactor into new loop that monitors thread health
    for handle in threads {
        handle.join().expect("error in pipeline thread");
    }

    Ok(())
}
