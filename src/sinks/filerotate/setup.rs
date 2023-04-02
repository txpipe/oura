use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
    ContentLimit, FileRotate,
};
use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;
use std::path::PathBuf;

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

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let output_path = match &self.output_path {
            Some(x) => PathBuf::try_from(x).map_err(Error::config)?,
            None => ctx.current_dir.clone(),
        };

        let suffix_scheme = AppendTimestamp::default(FileLimit::MaxFiles(
            self.max_total_files.unwrap_or(DEFAULT_MAX_TOTAL_FILES),
        ));

        let content_limit = ContentLimit::BytesSurpassed(
            self.max_bytes_per_file
                .unwrap_or(DEFAULT_MAX_BYTES_PER_FILE),
        );

        let compression = if let Some(true) = self.compress_files {
            Compression::OnRotate(2)
        } else {
            Compression::None
        };

        #[cfg(unix)]
        let writer = FileRotate::new(output_path, suffix_scheme, content_limit, compression, None);

        #[cfg(not(unix))]
        let writer = FileRotate::new(output_path, suffix_scheme, content_limit, compression);

        let worker = Worker {
            writer,
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(Bootstrapper(worker))
    }
}
