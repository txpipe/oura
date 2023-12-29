use std::io::Stdout;
use std::io::Write;

use gasket::framework::*;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::framework::*;

pub struct Worker {
    stdout: Stdout,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Self {
            stdout: std::io::stdout(),
        })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point();
        let json = JsonValue::from(unit.clone());

        self.stdout
            .write_all(json.to_string().as_bytes())
            .and_then(|_| self.stdout.write_all(b"\n"))
            .or_retry()?;

        stage.ops_count.inc(1);

        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-stdout", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config;

impl Config {
    pub fn bootstrapper(self, _: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
