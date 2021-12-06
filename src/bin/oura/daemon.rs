use std::sync::mpsc::{Sender, Receiver};

use clap::{ArgMatches, value_t};
use config::{Config, ConfigError, File, Environment};
use log::debug;
use oura::framework::{SourceConfig, BootstrapResult, SinkConfig};
use oura::ports::Event;
use oura::sources::chain::Config as NodeConfig;
use oura::sinks::terminal::Config as TerminalConfig;
use serde_derive::Deserialize;

use crate::Error;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Source {
    Node(NodeConfig),
}

impl SourceConfig for Source {
    fn bootstrap(&self, output: Sender<Event>) -> BootstrapResult {
        match self {
            Source::Node(c) => c.bootstrap(output),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Sink {
    Terminal(TerminalConfig)
}

impl SinkConfig for Sink {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult {
        match self {
            Sink::Terminal(c) => c.bootstrap(input),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigRoot {
    source: Source,
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
        s.merge(Environment::with_prefix("OURA"))?;

        s.try_into()
    }
}


pub fn run(args: &ArgMatches) -> Result<(), Error> {
    let explicit_config = match args.is_present("config") {
        true => Some(value_t!(args, "config", String)?),
        false => None,
    };

    let root = ConfigRoot::new(explicit_config)?;

    debug!("daemon starting with this config: {:?}", root);
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    let source = root.source.bootstrap(tx)?;
    let sink = root.sink.bootstrap(rx)?;

    // TODO: refactor into new loop that monitors thread health
    sink.join().map_err(|_| "error in sink thread")?;
    source.join().map_err(|_| "error in source thread")?;

    Ok(())
}
