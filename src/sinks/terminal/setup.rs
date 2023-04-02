use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;
use std::io::stdout;

use crate::framework::*;

use super::run::Worker;

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether =
            gasket::runtime::spawn_stage(self.0, gasket::runtime::Policy::default(), Some("sink"));

        Ok(vec![worker_tether])
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub throttle_min_span_millis: Option<u64>,
    pub wrap: Option<bool>,
    pub adahandle_policy: Option<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker {
            stdout: stdout(),
            throttle: self.throttle_min_span_millis.into(),
            wrap: self.wrap.unwrap_or(false),
            adahandle_policy: self.adahandle_policy,
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: ctx.cursor.clone(),
        };

        Ok(Bootstrapper(worker))
    }
}
