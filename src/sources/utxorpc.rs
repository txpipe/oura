use futures::StreamExt;
use gasket::framework::*;

use pallas::ledger::traverse::MultiEraBlock;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tonic::transport::Channel;
use tonic::Streaming;
use tracing::{debug, error};

use utxorpc::proto::sync::v1::any_chain_block::Chain;
use utxorpc::proto::sync::v1::chain_sync_service_client::ChainSyncServiceClient;
use utxorpc::proto::sync::v1::follow_tip_response::Action;
use utxorpc::proto::sync::v1::{BlockRef, DumpHistoryRequest, FollowTipRequest, FollowTipResponse};

use crate::framework::*;

fn point_to_blockref(point: Point) -> Option<BlockRef> {
    match point {
        Point::Origin => None,
        Point::Specific(slot, hash) => Some(BlockRef {
            index: slot,
            hash: hash.into(),
        }),
    }
}

pub struct Worker {
    client: ChainSyncServiceClient<Channel>,
    stream: Option<Streaming<FollowTipResponse>>,
    intersect: Option<BlockRef>,
    max_items_per_page: u32,
}

impl Worker {
    async fn process_next(&self, stage: &mut Stage, action: &Action) -> Result<(), WorkerError> {
        match action {
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
                    .send(ChainEvent::Reset(Point::new(reset.index, reset.hash.to_vec())).into())
                    .await
                    .or_panic()?;

                stage.chain_tip.set(reset.index as i64);
            }
        }

        Ok(())
    }

    async fn next_stream(&mut self) -> Result<WorkSchedule<Vec<Action>>, WorkerError> {
        if self.stream.is_none() {
            let stream = self
                .client
                .follow_tip(FollowTipRequest::default())
                .await
                .or_restart()?
                .into_inner();

            self.stream = Some(stream);
        }

        let result = self.stream.as_mut().unwrap().next().await;

        if result.is_none() {
            return Ok(WorkSchedule::Idle);
        }

        let result = result.unwrap();
        if let Err(err) = result {
            error!("{err}");
            return Err(WorkerError::Retry);
        }

        let response: FollowTipResponse = result.unwrap();
        if response.action.is_none() {
            return Ok(WorkSchedule::Idle);
        }

        let action = response.action.unwrap();

        Ok(WorkSchedule::Unit(vec![action]))
    }

    async fn next_dump_history(&mut self) -> Result<WorkSchedule<Vec<Action>>, WorkerError> {
        let dump_history_request = DumpHistoryRequest {
            start_token: self.intersect.clone(),
            max_items: self.max_items_per_page,
            ..Default::default()
        };

        let result = self
            .client
            .dump_history(dump_history_request)
            .await
            .or_restart()?
            .into_inner();

        self.intersect = result.next_token;

        if !result.block.is_empty() {
            let actions: Vec<Action> = result.block.into_iter().map(Action::Apply).collect();
            return Ok(WorkSchedule::Unit(actions));
        }

        Ok(WorkSchedule::Idle)
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let client = ChainSyncServiceClient::connect(stage.config.url.clone())
            .await
            .or_panic()?;

        let intersect: Vec<_> = if stage.breadcrumbs.is_empty() {
            stage.intersect.points().unwrap_or_default()
        } else {
            stage.breadcrumbs.points()
        };

        let intersect = intersect
            .into_iter()
            .map(point_to_blockref)
            .flatten()
            .collect::<Vec<_>>()
            .pop();

        let max_items_per_page = stage.config.max_items_per_page.unwrap_or(20);

        Ok(Self {
            client,
            stream: None,
            max_items_per_page,
            intersect,
        })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Vec<Action>>, WorkerError> {
        if self.intersect.is_some() {
            return self.next_dump_history().await;
        }

        self.next_stream().await
    }

    async fn execute(&mut self, unit: &Vec<Action>, stage: &mut Stage) -> Result<(), WorkerError> {
        for action in unit {
            self.process_next(stage, action).await.or_retry()?;
        }

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source-utxorpc", unit = "Vec<Action>", worker = "Worker")]
pub struct Stage {
    config: Config,
    breadcrumbs: Breadcrumbs,
    intersect: IntersectConfig,
    pub output: SourceOutputPort,
    #[metric]
    ops_count: gasket::metrics::Counter,
    #[metric]
    chain_tip: gasket::metrics::Gauge,
}

#[derive(Deserialize)]
pub struct Config {
    url: String,
    max_items_per_page: Option<u32>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            breadcrumbs: ctx.breadcrumbs.clone(),
            intersect: ctx.intersect.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
