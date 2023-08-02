use futures::StreamExt;
use gasket::framework::*;

use serde::Deserialize;
use tonic::Streaming;
use tracing::{debug, error};

use utxorpc::proto::sync::v1::chain_sync_service_client::ChainSyncServiceClient;
use utxorpc::proto::sync::v1::{follow_tip_response, FollowTipRequest, FollowTipResponse};

use crate::framework::*;

pub struct Worker {
    stream: Streaming<FollowTipResponse>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let mut client = ChainSyncServiceClient::connect(stage.config.url.clone())
            .await
            .or_panic()?;

        // TODO: configure intersect

        let stream = client
            .follow_tip(FollowTipRequest::default())
            .await
            .or_restart()?
            .into_inner();

        Ok(Self { stream })
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<follow_tip_response::Action>, WorkerError> {
        let result = self.stream.next().await;
        if result.is_none() {
            return Ok(WorkSchedule::Done);
        }

        let result = result.unwrap();
        if let Err(err) = result {
            error!("{err}");
            return Err(WorkerError::Retry);
        }

        let response: FollowTipResponse = result.unwrap();
        if response.action.is_none() {
            return Ok(WorkSchedule::Done);
        }

        let action = response.action.unwrap();
        Ok(WorkSchedule::Unit(action))
    }

    async fn execute(
        &mut self,
        unit: &follow_tip_response::Action,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        match unit {
            follow_tip_response::Action::Apply(block) => {
                debug!("APPLY {:?}", block)
            }
            utxorpc::proto::sync::v1::follow_tip_response::Action::Undo(any) => {
                debug!("UNDO {:?}", any)
            }
            utxorpc::proto::sync::v1::follow_tip_response::Action::Reset(reset) => {
                debug!("RESET {:?}", reset)
            }
        }
        Ok(())
    }
}

#[derive(Stage)]
#[stage(
    name = "source-utxorpc",
    unit = "follow_tip_response::Action",
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,
}

#[derive(Deserialize)]
pub struct Config {
    url: String,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
