use gasket::framework::*;
use serde::Deserialize;
use tracing::error;

use crate::framework::*;

use self::eval::{MatchOutcome, Predicate};

mod address;
mod eval;

#[derive(Stage)]
#[stage(name = "match_tx", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    predicate: Predicate,
    skip_uncertain: bool,

    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Worker {}
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let out = match unit {
        ChainEvent::Apply(point, record) => match record {
            Record::ParsedTx(tx) => {
                match eval::eval(record, &stage.predicate) {
                    MatchOutcome::Positive => {
                        Ok(Some(unit.to_owned()))
                    }
                    MatchOutcome::Negative => {
                        Ok(None)
                    }
                    MatchOutcome::Uncertain => {
                        if stage.skip_uncertain {

                        Ok(None)
                        } else {
                            Err(WorkerError::Panic)
                        }
                    }
                }
            },
            _ => {
                error!("The match_tx filter is valid only with the ParsedTx record");
                Err(WorkerError::Panic)
            }
        },
        _ => Ok(Some(unit.to_owned()))
    }?;

    stage.ops_count.inc(1);

    out
});

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
