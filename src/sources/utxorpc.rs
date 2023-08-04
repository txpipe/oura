use futures::StreamExt;
use gasket::framework::*;

use pallas::ledger::traverse::MultiEraBlock;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tonic::Streaming;
use tracing::{debug, error};

use utxorpc::proto::sync::v1::any_chain_block::Chain;
use utxorpc::proto::sync::v1::chain_sync_service_client::ChainSyncServiceClient;
use utxorpc::proto::sync::v1::follow_tip_response::Action;
use utxorpc::proto::sync::v1::{FollowTipRequest, FollowTipResponse};

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

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Action>, WorkerError> {
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

    async fn execute(&mut self, unit: &Action, stage: &mut Stage) -> Result<(), WorkerError> {
        match unit {
            Action::Apply(block) => {
                if let Some(chain) = &block.chain {
                    match chain {
                        Chain::Cardano(block) => {
                            if block.body.is_some() {
                                let header = block.header.as_ref().unwrap();

                                let block = block.body.as_ref().unwrap();

                                for tx in block.tx.clone() {
                                    let evt = ChainEvent::Apply(
                                        Point::Specific(header.slot, header.hash.to_vec()),
                                        Record::ParsedTx(tx),
                                    );

                                    stage.output.send(evt.into()).await.or_panic()?;
                                    stage.chain_tip.set(header.slot as i64);
                                }
                            }
                        }
                        Chain::Raw(bytes) => {
                            let block = MultiEraBlock::decode(bytes).or_panic()?;

                            let evt = ChainEvent::Apply(
                                Point::Specific(block.slot(), block.hash().to_vec()),
                                Record::CborBlock(bytes.to_vec()),
                            );

                            stage.output.send(evt.into()).await.or_panic()?;
                            stage.chain_tip.set(block.slot() as i64);
                        }
                    }
                }
            }
            Action::Undo(block) => {
                if let Some(chain) = &block.chain {
                    match chain {
                        Chain::Cardano(block) => {
                            if block.body.is_some() {
                                let header = block.header.as_ref().unwrap();

                                let block = block.body.as_ref().unwrap();

                                for tx in block.tx.clone() {
                                    let evt = ChainEvent::Undo(
                                        Point::Specific(header.slot, header.hash.to_vec()),
                                        Record::ParsedTx(tx),
                                    );

                                    stage.output.send(evt.into()).await.or_panic()?;
                                    stage.chain_tip.set(header.slot as i64);
                                }
                            }
                        }
                        Chain::Raw(bytes) => {
                            let block = MultiEraBlock::decode(bytes).or_panic()?;

                            let evt = ChainEvent::Undo(
                                Point::Specific(block.slot(), block.hash().to_vec()),
                                Record::CborBlock(bytes.to_vec()),
                            );

                            stage.output.send(evt.into()).await.or_panic()?;
                            stage.chain_tip.set(block.slot() as i64);
                        }
                    }
                }
            }
            Action::Reset(reset) => {
                stage
                    .output
                    .send(ChainEvent::reset(Point::new(reset.index, reset.hash.to_vec())).into())
                    .await
                    .or_panic()?;

                stage.chain_tip.set(reset.index as i64);
            }
        }
        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source-utxorpc", unit = "Action", worker = "Worker")]
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
    pub fn bootstrapper(self, _: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
