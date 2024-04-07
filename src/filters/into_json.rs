//! A noop filter used as example and placeholder for other filters

use gasket::framework::*;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::framework::*;

#[derive(Default, Stage)]
#[stage(name = "into-json", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Self
    }
}

gasket::impl_mapper!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let out = unit.clone().map_record(|r| Record::GenericJson(JsonValue::from(r)));
    stage.ops_count.inc(1);
    out
});

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        Ok(Stage::default())
    }
}
