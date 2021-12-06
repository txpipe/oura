use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use crate::ports::Event;

pub type Error = Box<dyn std::error::Error>;

pub type BootstrapResult = Result<JoinHandle<()>, Error>;

pub trait SourceConfig {
    fn bootstrap(&self, output: Sender<Event>) -> BootstrapResult;
}

pub trait SinkConfig {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult;
}
