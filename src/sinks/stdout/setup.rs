use std::io::stdout;

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
};

use super::run::jsonl_writer_loop;

#[derive(Debug, Deserialize, Clone)]
pub enum Format {
    JSONL,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Output {
    Stdout,
    FileRotate,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub format: Option<Format>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let format = self.inner.format.as_ref().cloned().unwrap_or(Format::JSONL);

        let mut output = stdout();

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || match format {
            Format::JSONL => {
                jsonl_writer_loop(input, &mut output, utils).expect("writer sink loop failed")
            }
        });

        Ok(handle)
    }
}
