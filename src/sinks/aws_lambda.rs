use aws_config::BehaviorVersion;
use aws_sdk_lambda::{primitives::Blob, Client};
use aws_types::region::Region;
use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    client: Client,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(Region::new(stage.config.region.clone()))
            .load()
            .await;

        let client = Client::new(&aws_config);

        Ok(Self { client })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let payload = serde_json::Value::from(record.unwrap()).to_string();

        let req = self
            .client
            .invoke()
            .function_name(stage.config.function_name.clone())
            .payload(Blob::new(payload));

        req.send().await.or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-aws-lambda", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub region: String,
    pub function_name: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
