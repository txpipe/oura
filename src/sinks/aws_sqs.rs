use aws_sdk_sqs::Client;
use aws_types::region::Region;
use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    client: Client,
    group_id: Option<String>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let aws_config = aws_config::from_env()
            .region(Region::new(stage.config.region.clone()))
            .load()
            .await;

        let mut worker = Self {
            client: Client::new(&aws_config),
            group_id: None,
        };

        if stage.config.queue_url.clone().ends_with(".fifo") {
            worker.group_id = Some(
                stage
                    .config
                    .group_id
                    .clone()
                    .unwrap_or(String::from("oura-sink")),
            )
        }

        Ok(worker)
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

        let mut req = self
            .client
            .send_message()
            .queue_url(stage.config.queue_url.clone())
            .message_body(payload);

        if self.group_id.is_some() {
            req = req.set_message_group_id(self.group_id.clone())
        }

        req.send().await.or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-aws-sqs", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    cursor: Cursor,

    pub input: MapperInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub region: String,
    pub queue_url: String,
    pub group_id: Option<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(stage)
    }
}
