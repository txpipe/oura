use gasket::framework::*;
use itertools::Itertools;
use miette::{Context as _, IntoDiagnostic as _};
use mithril_client::{ClientBuilder, MessageBuilder, MithrilError, MithrilResult};
use pallas::{
    ledger::traverse::MultiEraBlock,
    network::miniprotocols::Point::{self, *},
};
use serde::Deserialize;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{info, warn};

use crate::framework::*;

struct Feedback {
    progress_logger: Arc<Mutex<ProgressLogger>>,
}

impl Feedback {
    fn new(log_interval: Duration) -> Self {
        Self {
            progress_logger: Arc::new(Mutex::new(ProgressLogger::new(log_interval))),
        }
    }

    fn log_progress(&self, downloaded_bytes: u64, size: u64) {
        if let Ok(mut logger) = self.progress_logger.lock() {
            logger.log(downloaded_bytes, size);
        }
    }
}

impl Default for Feedback {
    fn default() -> Self {
        Self::new(Duration::from_secs(10))
    }
}

struct ProgressLogger {
    last_log_time: Option<Instant>,
    log_interval: Duration,
    initial_logged: bool,
}

impl ProgressLogger {
    fn new(log_interval: Duration) -> Self {
        Self {
            last_log_time: None,
            log_interval,
            initial_logged: false,
        }
    }

    fn log(&mut self, downloaded_bytes: u64, size: u64) {
        let now = Instant::now();
        let percentage = (downloaded_bytes as f64 / size as f64 * 100.0).round() as u64;

        if !self.initial_logged {
            info!(
                "Initial snapshot download progress: {}% ({}/{} bytes)",
                percentage, downloaded_bytes, size
            );
            self.initial_logged = true;
            self.last_log_time = Some(now);
            return;
        }

        if downloaded_bytes == size {
            info!(
                "Snapshot download complete: 100% ({}/{} bytes)",
                downloaded_bytes, size
            );
            self.last_log_time = Some(now);
            return;
        }

        if let Some(last_time) = self.last_log_time {
            if now.duration_since(last_time) >= self.log_interval {
                info!(
                    "Snapshot download progress: {}% ({}/{} bytes)",
                    percentage, downloaded_bytes, size
                );
                self.last_log_time = Some(now);
            }
        }
    }
}

#[async_trait::async_trait]
impl mithril_client::feedback::FeedbackReceiver for Feedback {
    async fn handle_event(&self, event: mithril_client::feedback::MithrilEvent) {
        match event {
            mithril_client::feedback::MithrilEvent::SnapshotDownloadStarted { .. } => {
                info!("snapshot download started");
            }
            mithril_client::feedback::MithrilEvent::SnapshotDownloadProgress {
                downloaded_bytes,
                size,
                ..
            } => {
                self.log_progress(downloaded_bytes, size);
            }
            mithril_client::feedback::MithrilEvent::SnapshotDownloadCompleted { .. } => {
                info!("snapshot download completed");
            }
            mithril_client::feedback::MithrilEvent::CertificateChainValidationStarted {
                ..
            } => {
                info!("certificate chain validation started");
            }
            mithril_client::feedback::MithrilEvent::CertificateValidated {
                certificate_hash: hash,
                ..
            } => {
                info!("certificate validated: {hash}");
            }
            mithril_client::feedback::MithrilEvent::CertificateChainValidated { .. } => {
                info!("certificate chain validation completed");
            }
            _ => {}
        }
    }
}

async fn fetch_snapshot(config: &Config, feedback: Arc<Feedback>) -> MithrilResult<()> {
    let client = ClientBuilder::aggregator(&config.aggregator, &config.genesis_key)
        .add_feedback_receiver(feedback)
        .build()?;

    let snapshots = client.cardano_database().list().await?;

    let last_digest = snapshots
        .first()
        .ok_or(MithrilError::msg("no snapshot available"))?
        .digest
        .as_ref();

    let snapshot = client
        .cardano_database()
        .get(last_digest)
        .await?
        .ok_or(MithrilError::msg("no snapshot available"))?;

    let target_directory = Path::new(&config.snapshot_download_dir);

    client
        .cardano_database()
        .download_unpack(&snapshot, target_directory)
        .await?;

    if let Err(e) = client.cardano_database().add_statistics(&snapshot).await {
        warn!("failed incrementing snapshot download statistics: {:?}", e);
    }

    let certificate = if config.skip_validation {
        client
            .certificate()
            .get(&snapshot.certificate_hash)
            .await?
            .ok_or(MithrilError::msg("certificate for snapshot not found"))?
    } else {
        client
            .certificate()
            .verify_chain(&snapshot.certificate_hash)
            .await?
    };

    let message = MessageBuilder::new()
        .compute_snapshot_message(&certificate, target_directory)
        .await?;

    assert!(certificate.match_message(&message));

    Ok(())
}

fn get_starting_points(
    dir: &Path,
    config: &IntersectConfig,
) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    match config {
        IntersectConfig::Tip => pallas::storage::hardano::immutable::get_tip(dir)?
            .map_or(Ok(vec![Point::Origin]), |point| Ok(vec![point])),
        IntersectConfig::Origin => Ok(vec![Point::Origin]),
        IntersectConfig::Point(slot, hash) => {
            let hash_bytes = hex::decode(hash)?;
            Ok(vec![Point::Specific(*slot, hash_bytes)])
        }
        IntersectConfig::Breadcrumbs(points) => points
            .iter()
            .map(|(slot, hash)| {
                let hash_bytes = hex::decode(hash)?;
                Ok(Point::Specific(*slot, hash_bytes))
            })
            .collect(),
    }
}

fn read_blocks_with_config(
    immutable_path: &Path,
    config: &IntersectConfig,
) -> Result<
    Box<dyn Iterator<Item = pallas::storage::hardano::immutable::FallibleBlock> + Send + Sync>,
    WorkerError,
> {
    let starting_points =
        get_starting_points(immutable_path, config).map_err(|_| WorkerError::Panic)?;

    for point in starting_points {
        match pallas::storage::hardano::immutable::read_blocks_from_point(immutable_path, point) {
            Ok(iter) => return Ok(iter),
            Err(_) => continue,
        }
    }

    // If all points fail (or if the list was empty), try from Origin
    pallas::storage::hardano::immutable::read_blocks_from_point(immutable_path, Point::Origin)
        .map_err(|_| WorkerError::Panic)
}

#[derive(Stage)]
#[stage(name = "source", unit = "()", worker = "Worker")]
pub struct Stage {
    config: Config,
    intersect: IntersectConfig,
    pub output: SourceOutputPort,
}

pub struct Worker {
    config: Config,
    is_bootstrapped: bool,
}

impl Worker {}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let feedback = Arc::new(Feedback::default());
        let target_directory = Path::new(&stage.config.snapshot_download_dir);

        if !target_directory.exists() {
            std::fs::create_dir_all(target_directory)
                .map_err(|err| miette::miette!(err.to_string()))
                .context(format!(
                    "Failed to create directory: {}",
                    target_directory.display()
                ))
                .map_err(|_| WorkerError::Panic)?;
        }

        // Check if the directory is empty
        let is_dir_empty = target_directory
            .read_dir()
            .map_err(|err| miette::miette!(err.to_string()))
            .context("Failed to read target directory")
            .map_err(|_| WorkerError::Panic)?
            .next()
            .is_none();

        if is_dir_empty {
            // Directory is empty, fetch the snapshot
            fetch_snapshot(&stage.config, feedback.clone())
                .await
                .map_err(|err| miette::miette!(err.to_string()))
                .context("fetching and validating mithril snapshot")
                .map_err(|_| WorkerError::Panic)?;
        } else {
            println!("Snapshot directory is not empty. Assuming existing snapshot data.");
            // @TODO add data validation (?) or just assume it's correct
        }

        Ok(Self {
            config: stage.config.clone(),
            is_bootstrapped: false, // Set to true since we either fetched or found existing data
        })
    }

    async fn schedule(&mut self, _stage: &mut Stage) -> Result<WorkSchedule<()>, WorkerError> {
        if self.is_bootstrapped {
            Ok(WorkSchedule::Done)
        } else {
            Ok(WorkSchedule::Unit(()))
        }
    }

    async fn execute(&mut self, _unit: &(), stage: &mut Stage) -> Result<(), WorkerError> {
        let immutable_path = Path::new(&self.config.snapshot_download_dir).join("immutable");

        let iter = read_blocks_with_config(&immutable_path, &stage.intersect)
            .into_diagnostic()
            .context("reading immutable db")
            .map_err(|_| WorkerError::Panic)?;

        for chunk in iter.chunks(100).into_iter() {
            let bodies: Vec<_> = chunk
                .try_collect()
                .into_diagnostic()
                .context("reading block data")
                .map_err(|_| WorkerError::Panic)?;

            let blocks: Vec<(Point, Vec<u8>)> = bodies
                .iter()
                .map(|b| {
                    let blockd = MultiEraBlock::decode(b)
                        .into_diagnostic()
                        .context("decoding block cbor")
                        .unwrap();
                    (Specific(blockd.slot(), blockd.hash().to_vec()), b.clone())
                })
                .collect();

            for (point, block) in blocks {
                let event = ChainEvent::Apply(point, Record::CborBlock(block));
                stage.output.send(event.into()).await.or_panic()?;
            }
        }

        self.is_bootstrapped = true;

        Ok(())
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    aggregator: String,
    genesis_key: String,
    snapshot_download_dir: String,
    skip_validation: bool,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            intersect: ctx.intersect.clone(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
