use std::collections::HashMap;

use gasket::framework::*;
use pallas::network::miniprotocols::{chainsync, Point};
use serde::Deserialize;
use tracing::{debug, info};

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "rollback-buffer", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    min_depth: usize,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker {
    buffer: chainsync::RollbackBuffer,
    events: HashMap<Point, Record>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Default::default())
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        match unit {
            ChainEvent::Apply(point, record) => {
                self.events.insert(point.clone(), record.clone());
                self.buffer.roll_forward(point.clone());

                let ready = self.buffer.pop_with_depth(stage.min_depth);
                for point in ready {
                    if let Some(record) = self.events.remove(&point) {
                        stage
                            .output
                            .send(ChainEvent::apply(point, record))
                            .await
                            .or_panic()?;
                    }
                }
            }
            ChainEvent::Undo(point, _) => match self.buffer.roll_back(point) {
                chainsync::RollbackEffect::Handled => {
                    debug!(?point, "handled rollback within buffer");
                    self.events
                        .retain(|x, _| x.slot_or_default() <= point.slot_or_default());
                }
                chainsync::RollbackEffect::OutOfScope => {
                    debug!("rollback out of buffer scope, sending event down the pipeline");
                    self.events.clear();
                    stage
                        .output
                        .send(ChainEvent::reset(point.clone()))
                        .await
                        .or_panic()?;
                }
            },
            ChainEvent::Reset(point) => {
                self.events.clear();
                stage
                    .output
                    .send(ChainEvent::reset(point.clone()))
                    .await
                    .or_panic()?;
            }
        };

        info!(
            "rollback buffer state, size: {}, oldest: {:?}, latest: {:?}",
            self.buffer.size(),
            self.buffer.oldest(),
            self.buffer.latest(),
        );

        stage.ops_count.inc(1);

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub min_depth: usize,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        info!(capacity = ?self.min_depth, "buffer filter capacity");

        let stage = Stage {
            min_depth: self.min_depth,
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
