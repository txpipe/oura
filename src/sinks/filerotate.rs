use std::io::Write;
use std::path::PathBuf;

use file_rotate::compression::Compression;
use file_rotate::suffix::AppendTimestamp;
use file_rotate::suffix::FileLimit;
use file_rotate::ContentLimit;
use file_rotate::FileRotate;
use gasket::framework::*;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::framework::*;

pub struct Worker {
    writer: FileRotate<AppendTimestamp>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let output_path = match &stage.config.output_path {
            Some(x) => PathBuf::try_from(x).map_err(Error::config).or_panic()?,
            None => stage.current_dir.clone(),
        };

        let suffix_scheme = AppendTimestamp::default(FileLimit::MaxFiles(
            stage
                .config
                .max_total_files
                .unwrap_or(DEFAULT_MAX_TOTAL_FILES),
        ));

        let content_limit = ContentLimit::BytesSurpassed(
            stage
                .config
                .max_bytes_per_file
                .unwrap_or(DEFAULT_MAX_BYTES_PER_FILE),
        );

        let compression = if let Some(true) = stage.config.compress_files {
            Compression::OnRotate(2)
        } else {
            Compression::None
        };

        #[cfg(unix)]
        let writer = FileRotate::new(output_path, suffix_scheme, content_limit, compression, None);

        #[cfg(not(unix))]
        let writer = FileRotate::new(output_path, suffix_scheme, content_limit, compression);

        Ok(Self { writer })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point();
        let json = JsonValue::from(unit.clone());

        self.writer
            .write_all(json.to_string().as_bytes())
            .and_then(|_| self.writer.write_all(b"\n"))
            .or_retry()?;

        stage.ops_count.inc(1);

        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "filter", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    current_dir: PathBuf,
    cursor: Cursor,

    pub input: MapperInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
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
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            current_dir: ctx.current_dir.clone(),
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(stage)
    }
}
