use std::io::Write;
use std::path::PathBuf;

use file_rotate::compression::Compression;
use file_rotate::suffix::AppendTimestamp;
use file_rotate::suffix::FileLimit;
use file_rotate::ContentLimit;
use file_rotate::FileRotate;
use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value as JsonValue;

use crate::framework::*;

pub struct Worker {
    writer: FileRotate<AppendTimestamp>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = ChainEvent;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
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
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        let (point, json) = match unit {
            ChainEvent::Apply(point, record) => {
                let json = json!({ "event": "apply", "record": JsonValue::from(record.clone()) });
                (point, json)
            }
            ChainEvent::Undo(point, record) => {
                let json = json!({ "event": "undo", "record": JsonValue::from(record.clone()) });
                (point, json)
            }
            ChainEvent::Reset(point) => {
                let json_point = match &point {
                    pallas::network::miniprotocols::Point::Origin => JsonValue::from("origin"),
                    pallas::network::miniprotocols::Point::Specific(slot, hash) => {
                        json!({ "slot": slot, "hash": hex::encode(hash)})
                    }
                };

                let json = json!({ "event": "reset", "point": json_point });
                (point, json)
            }
        };

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

pub struct Stage {
    config: Config,
    current_dir: PathBuf,
    cursor: Cursor,
    ops_count: gasket::metrics::Counter,
    latest_block: gasket::metrics::Gauge,
    input: MapperInputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "sink"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
        registry.track_gauge("latest_block", &self.latest_block);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

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
