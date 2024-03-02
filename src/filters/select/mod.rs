use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

use self::eval::{MatchOutcome, Predicate};

mod eval;

#[derive(Stage)]
#[stage(name = "select", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    predicate: Predicate,
    skip_uncertain: bool,

    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

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
        let is_match = match unit {
            ChainEvent::Apply(_, r) => eval::eval(r, &stage.predicate),
            ChainEvent::Undo(_, r) => eval::eval(r, &stage.predicate),
            ChainEvent::Reset(_) => MatchOutcome::Positive,
        };

        match is_match {
            MatchOutcome::Positive => stage.output.send(unit.clone().into()).await.or_panic()?,
            MatchOutcome::Negative => (),
            MatchOutcome::Uncertain => {
                if !stage.skip_uncertain {
                    return Err(WorkerError::Panic);
                }
            }
        };

        stage.ops_count.inc(1);

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub predicate: Predicate,
    pub skip_uncertain: bool,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            predicate: self.predicate,
            skip_uncertain: self.skip_uncertain,
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
