use aws_sdk_s3::{primitives::ByteStream, Client};
use aws_types::region::Region;
use gasket::framework::*;
use serde::Deserialize;
use serde_json::json;

use crate::framework::*;

pub struct Worker {
    client: Client,
    prefix: String,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let aws_config = aws_config::from_env()
            .region(Region::new(stage.config.region.clone()))
            .load()
            .await;

        let client = Client::new(&aws_config);
        let prefix = stage
            .config
            .prefix
            .clone()
            .and_then(|p| Some(format!("{p}/")))
            .unwrap_or_default();

        Ok(Self { client, prefix })
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

        let (payload, content_type) = match record.clone().unwrap() {
            Record::CborBlock(cbor) | Record::CborTx(cbor) => (cbor, "application/cbor"),
            Record::OuraV1Event(event) => (json!(event).to_string().into(), "application/json"),
            Record::ParsedTx(tx) => (json!(tx).to_string().into(), "application/json"),
            Record::GenericJson(value) => (value.to_string().into(), "application/json"),
        };

        let key = format!("{}{}", self.prefix, point.slot_or_default());

        self.client
            .put_object()
            .bucket(&stage.config.bucket)
            .key(key)
            .body(ByteStream::from(payload))
            .content_type(content_type)
            .send()
            .await
            .or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-aws-s3", unit = "ChainEvent", worker = "Worker")]
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
    pub bucket: String,
    pub prefix: Option<String>,
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
