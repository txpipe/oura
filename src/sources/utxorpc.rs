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

pub struct Worker {
    client: ChainSyncServiceClient<Channel>,
    stream: Option<Streaming<FollowTipResponse>>,
    block_ref: Option<BlockRef>,
    max_items: u32,
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
        let mut dump_history_request = DumpHistoryRequest::default();
        dump_history_request.start_token = self.block_ref.clone();
        dump_history_request.max_items = self.max_items;

        let result = self
            .client
            .dump_history(dump_history_request)
            .await
            .or_restart()?
            .into_inner();

        self.block_ref = result.next_token;

        if !result.block.is_empty() {
            let actions: Vec<Action> = result.block.into_iter().map(|b| Action::Apply(b)).collect();
            return Ok(WorkSchedule::Unit(actions));
        }

        return Ok(WorkSchedule::Idle);
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let client = ChainSyncServiceClient::connect(stage.config.url.clone())
            .await
            .or_panic()?;

        let mut point: Option<(u64, Vec<u8>)> = match stage.intersect.clone() {
            IntersectConfig::Point(slot, hash) => Some((slot, hash.into())),
            _ => None,
        };

        if let Some(latest_point) = stage.cursor.latest_known_point() {
            point = match latest_point {
                Point::Specific(slot, hash) => Some((slot, hash)),
                _ => None,
            };
        }

        let block_ref = if let Some((slot, hash)) = point {
            let mut block_ref = BlockRef::default();
            block_ref.index = slot;
            block_ref.hash = hash.into();
            Some(block_ref)
        } else {
            None
        };

        let max_items = stage.config.max_items.unwrap_or(20);

        Ok(Self {
            client,
            stream: None,
            max_items,
            block_ref,
        })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Vec<Action>>, WorkerError> {
        if self.block_ref.is_some() {
            return Ok(self.next_dump_history().await?);
        }

        Ok(self.next_stream().await?)
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
    cursor: Cursor,
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
    max_items: Option<u32>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            cursor: ctx.cursor.clone(),
            intersect: ctx.intersect.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
