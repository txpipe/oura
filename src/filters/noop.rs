//! A noop filter used as example and placeholder for other filters

use std::thread;

use serde::Deserialize;

use crate::pipelining::{
    new_inter_stage_channel, FilterProvider, PartialBootstrapResult, StageReceiver,
};

#[derive(Debug, Deserialize)]
pub struct Config {}

impl FilterProvider for Config {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let handle = thread::spawn(move || {
            for msg in input.iter() {
                output_tx.send(msg).expect("error sending filter message");
            }
        });

        Ok((handle, output_rx))
    }
}
