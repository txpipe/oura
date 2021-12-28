//! A noop filter used as example and placeholder for other filters

use std::{sync::mpsc::Receiver, thread};

use serde_derive::Deserialize;

use crate::framework::{Event, FilterConfig, PartialBootstrapResult};

#[derive(Debug, Deserialize)]
pub struct Config {}

impl FilterConfig for Config {
    fn bootstrap(&self, input: Receiver<Event>) -> PartialBootstrapResult {
        let (output_tx, output_rx) = std::sync::mpsc::channel();

        let handle = thread::spawn(move || {
            loop {
                let msg = input.recv().expect("error receiving message");
                output_tx.send(msg).expect("error sending filter message");
            }
        });

        Ok((handle, output_rx))
    }
}
