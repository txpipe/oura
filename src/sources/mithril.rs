use miette::{Context as _, IntoDiagnostic as _};
use pallas::{ledger::traverse::MultiEraBlock, network::miniprotocols::Point::{self, *}};
use serde::Deserialize;
use gasket::framework::*;
use mithril_client::{ClientBuilder, MessageBuilder, MithrilError, MithrilResult};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tracing::warn;
use std::{path::Path, sync::Arc};
use itertools::Itertools;

use crate::framework::*;

struct Feedback {
    _multi: MultiProgress,
    download_pb: ProgressBar,
    validate_pb: ProgressBar,
}

impl Feedback {
    fn indeterminate_progress_bar(owner: &mut MultiProgress) -> ProgressBar {
        let pb = ProgressBar::new_spinner();

        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}").unwrap(),
        );

        owner.add(pb)
    }

    fn bytes_progress_bar(owner: &mut MultiProgress) -> ProgressBar {
        let pb = ProgressBar::new_spinner();

        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} (eta: {eta}) {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        owner.add(pb)
    }
}

impl Default for Feedback {
    fn default() -> Self {
        let mut multi = MultiProgress::new();

        Self {
            download_pb: Self::bytes_progress_bar(&mut multi),
            validate_pb: Self::indeterminate_progress_bar(&mut multi),
            _multi: multi,
        }
    }
}

#[async_trait::async_trait]
impl mithril_client::feedback::FeedbackReceiver for Feedback {
    async fn handle_event(&self, event: mithril_client::feedback::MithrilEvent) {
        match event {
            mithril_client::feedback::MithrilEvent::SnapshotDownloadStarted { .. } => {
                self.download_pb.set_message("snapshot download started")
            }
            mithril_client::feedback::MithrilEvent::SnapshotDownloadProgress {
                downloaded_bytes,
                size,
                ..
            } => {
                self.download_pb.set_length(size);
                self.download_pb.set_position(downloaded_bytes);
                self.download_pb.set_message("downloading Mithril snapshot");
            }
            mithril_client::feedback::MithrilEvent::SnapshotDownloadCompleted { .. } => {
                self.download_pb.set_message("snapshot download completed");
            }
            mithril_client::feedback::MithrilEvent::CertificateChainValidationStarted {
                ..
            } => {
                self.validate_pb
                    .set_message("certificate chain validation started");
            }
            mithril_client::feedback::MithrilEvent::CertificateValidated {
                certificate_hash: hash,
                ..
            } => {
                self.validate_pb
                    .set_message(format!("validating cert: {hash}"));
            }
            mithril_client::feedback::MithrilEvent::CertificateChainValidated { .. } => {
                self.validate_pb.set_message("certificate chain validated");
            }
        }
    }
}

async fn fetch_snapshot(
    config: &Config,
    feedback: Arc<Feedback>,
) -> MithrilResult<()> {
    let client = ClientBuilder::aggregator(&config.aggregator, &config.genesis_key)
        .add_feedback_receiver(feedback)
        .build()?;

    let snapshots = client.snapshot().list().await?;

    let last_digest = snapshots
        .first()
        .ok_or(MithrilError::msg("no snapshot available"))?
        .digest
        .as_ref();

    let snapshot = client
        .snapshot()
        .get(last_digest)
        .await?
        .ok_or(MithrilError::msg("no snapshot available"))?;

    let target_directory = Path::new(&config.snapshot_download_dir);

    client
        .snapshot()
        .download_unpack(&snapshot, target_directory)
        .await?;

    if let Err(e) = client.snapshot().add_statistics(&snapshot).await {
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

#[derive(Stage)]
#[stage(
    name = "source",
    unit = "()",
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    pub output: SourceOutputPort,
}

pub struct Worker {
    config: Config,
}

impl Worker {
}

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
        
        fetch_snapshot(&stage.config, feedback.clone())
            .await
            .map_err(|err| miette::miette!(err.to_string()))
            .context("fetching and validating mithril snapshot")
            .map_err(|_| WorkerError::Panic)?;

        Ok(Self {
            config: stage.config.clone(),
        })
    }

    async fn schedule(&mut self, _stage: &mut Stage) -> Result<WorkSchedule<()>, WorkerError> {
        Ok(WorkSchedule::Unit(()))
    }

    async fn execute(&mut self, _unit: &(), stage: &mut Stage) -> Result<(), WorkerError> {

        let immutable_path = Path::new(&self.config.snapshot_download_dir).join("immutable");

        let iter = pallas::storage::hardano::immutable::read_blocks(&immutable_path)
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
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
        };

        Ok(stage)
    }
}