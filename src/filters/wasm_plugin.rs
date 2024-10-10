//! A filter that maps records by calling custom WASM plugins

use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "filter-wasm", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    plugin: extism::Plugin,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

impl Stage {
    fn map_record(&mut self, r: Record) -> Result<Vec<Record>, Error> {
        let extism::convert::Json::<serde_json::Value>(output) = match r {
            Record::CborBlock(x) => self.plugin.call("map_cbor_block", x).unwrap(),
            Record::CborTx(_, x) => self.plugin.call("map_cbor_tx", x).unwrap(),
            Record::CborUtxo(_, x, spent) => self.plugin.call("map_cbor_utxo", x).unwrap(),
            Record::ParsedTx(x) => self
                .plugin
                .call("map_u5c_tx", extism::convert::Json(x))
                .unwrap(),
            Record::ParsedBlock(x) => self
                .plugin
                .call("map_u5c_block", extism::convert::Json(x))
                .unwrap(),
            Record::GenericJson(x) => self
                .plugin
                .call("map_json", extism::convert::Json(x))
                .unwrap(),
            Record::OuraV1Event(x) => self
                .plugin
                .call("map_json", extism::convert::Json(x))
                .unwrap(),
        };

        let output = match output {
            serde_json::Value::Null => vec![],
            serde_json::Value::Array(x) => x.into_iter().map(Record::GenericJson).collect(),
            x => vec![Record::GenericJson(x)],
        };

        Ok(output)
    }
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Self
    }
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
        let output = unit
            .clone()
            .try_map_record_to_many(|x| stage.map_record(x))
            .or_panic()?;

        for unit in output {
            stage.output.send(unit.clone().into()).await.or_panic()?;
            stage.ops_count.inc(1);
        }

        Ok(())
    }
}

#[derive(Default, Deserialize)]
pub struct Config {
    path: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let wasm = extism::Wasm::file(self.path);
        let manifest = extism::Manifest::new([wasm]);
        let plugin = extism::Plugin::new(&manifest, [], true).map_err(Error::custom)?;

        Ok(Stage {
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
            plugin,
        })
    }
}
