use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::error;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "filter-match-pattern", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    predicate: Predicate,

    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Worker::default()
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let out = match unit {
        ChainEvent::Apply(point, record) => match record {
            Record::ParsedTx(tx) => {
                if stage.predicate.tx_match(point, tx) {
                    Ok(Some(unit.to_owned()))
                } else {
                    Ok(None)
                }
            },
            _ => {
                error!("The MatchPattern filter is valid only with the ParsedTx record");
                Err(WorkerError::Panic)
            }
        },
        _ => Ok(Some(unit.to_owned()))
    }?;

    stage.ops_count.inc(1);

    out
});

#[derive(Deserialize, Clone)]
pub struct AddressPattern {
    pub exact_hex: Option<String>,
    pub exact_bech32: Option<String>,
    pub payment_hex: Option<String>,
    pub payment_bech32: Option<String>,
    pub stake_hex: Option<String>,
    pub stake_bech32: Option<String>,
    pub is_script: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct BlockPattern {
    pub slot_before: Option<u64>,
    pub slot_after: Option<u64>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Predicate {
    Block(BlockPattern),
}

impl Predicate {
    fn tx_match(&self, point: &Point, _: &ParsedTx) -> bool {
        match self {
            Predicate::Block(block_pattern) => self.slot_match(point, block_pattern),
        }
    }

    fn slot_match(&self, point: &Point, block_pattern: &BlockPattern) -> bool {
        if let Some(slot_after) = block_pattern.slot_after {
            if point.slot_or_default() <= slot_after {
                return false;
            }
        }

        if let Some(slot_before) = block_pattern.slot_before {
            if point.slot_or_default() >= slot_before {
                return false;
            }
        }

        true
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub predicate: Predicate,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            predicate: self.predicate,
            ops_count: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(stage)
    }
}
