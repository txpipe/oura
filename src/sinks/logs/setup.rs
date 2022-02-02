use file_rotate::{
    suffix::{FileLimit, TimestampSuffixScheme},
    ContentLimit, FileRotate,
};
use std::{io::Write, path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::WithUtils,
    Error,
};

use super::run::jsonl_writer_loop;

#[derive(Debug, Deserialize, Clone)]
pub enum Format {
    JSONL,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub output_format: Option<Format>,
    pub output_path: Option<String>,
    pub max_bytes_per_file: Option<usize>,
    pub max_total_files: Option<usize>,
    pub compress_files: Option<bool>,
}

const DEFAULT_MAX_BYTES_PER_FILE: usize = 50 * 1024 * 1024;
const DEFAULT_MAX_TOTAL_FILES: usize = 200;

fn build_witer(config: &Config) -> Result<impl Write, Error> {
    let output_path = match &config.output_path {
        Some(x) => PathBuf::from_str(x)?,
        None => std::env::current_dir()?,
    };

    let suffix_scheme = TimestampSuffixScheme::default(FileLimit::MaxFiles(
        config.max_total_files.unwrap_or(DEFAULT_MAX_TOTAL_FILES),
    ));

    let content_limit = ContentLimit::BytesSurpassed(
        config
            .max_bytes_per_file
            .unwrap_or(DEFAULT_MAX_BYTES_PER_FILE),
    );

    let compression = file_rotate::compression::Compression::OnRotate(2);

    let writer = FileRotate::new(output_path, suffix_scheme, content_limit, compression);

    Ok(writer)
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let mut output = build_witer(&self.inner)?;

        let format = self
            .inner
            .output_format
            .as_ref()
            .cloned()
            .unwrap_or(Format::JSONL);

        let utils = self.utils.clone();

        let handle = std::thread::spawn(move || match format {
            Format::JSONL => {
                jsonl_writer_loop(input, &mut output, utils).expect("logs sink loop failed")
            }
        });

        Ok(handle)
    }
}
