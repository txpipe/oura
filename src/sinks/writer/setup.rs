use std::io::stdout;

use serde::Deserialize;

use crate::pipelining::{BootstrapResult, SinkProvider, StageReceiver};

use super::run::consumer_loop;

#[derive(Debug, Deserialize, Clone)]
pub enum OutputFormat {
    JSONL,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub format: Option<OutputFormat>,
}

impl SinkProvider for Config {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let format = self
            .format
            .as_ref()
            .map(|x| x.clone())
            .unwrap_or(OutputFormat::JSONL);

        let mut output = stdout();

        let handle = std::thread::spawn(move || {
            consumer_loop(format, input, &mut output).expect("writer sink loop failed");
        });

        Ok(handle)
    }
}
