//! A noop filter used as example and placeholder for other filters

use std::thread;

use serde_derive::Deserialize;

use crate::framework::{new_inter_stage_channel, FilterConfig, PartialBootstrapResult, StageReceiver};

#[derive(Debug, Deserialize)]
pub struct Config {}

impl FilterConfig for Config {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let handle = thread::spawn(move || loop {
            let msg = input.recv().expect("error receiving message");
            output_tx.send(msg).expect("error sending filter message");
        });

        Ok((handle, output_rx))
    }
}
