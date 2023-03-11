use serde::Deserialize;
use std::io::stdout;

use crate::framework::*;

use super::run::Worker;

pub struct Runtime {
    worker_tether: gasket::runtime::Tether,
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &mut MapperInputPort {
        &mut self.0.input
    }

    pub fn borrow_output_port(&mut self) -> &mut MapperOutputPort {
        &mut self.0.output
    }

    pub fn spawn(self) -> Result<Runtime, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("sink_terminal"),
        );

        Ok(Runtime { worker_tether })
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
            msg_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
            cursor: ctx.cursor.clone(),
        };

        Ok(Bootstrapper(worker))
    }
}
