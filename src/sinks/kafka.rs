use std::time::Duration;

use gasket::framework::*;
use kafka::producer::{Producer, Record, RequiredAcks};
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    producer: Producer,
    partitioning: PartitionStrategy,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let mut builder = Producer::from_hosts(stage.config.brokers.clone());

        if let Some(timeout) = stage.config.ack_timeout_secs {
            builder = builder
                .with_ack_timeout(Duration::from_secs(timeout))
                .with_required_acks(RequiredAcks::One)
        };

        let producer = builder.create().or_panic()?;

        let partitioning = stage
            .config
            .paritioning
            .clone()
            .unwrap_or(PartitionStrategy::Random);

        Ok(Self {
            producer,
            partitioning,
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
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let payload = serde_json::to_vec(&serde_json::Value::from(record.unwrap())).or_panic()?;

        match self.partitioning {
            PartitionStrategy::ByBlock => {
                let slot = point.slot_or_default().to_be_bytes();
                let kafka_record = Record::from_key_value(&stage.config.topic, &slot[..], payload);
                self.producer.send(&kafka_record)
            }
            PartitionStrategy::Random => {
                let kafka_record = Record::from_value(&stage.config.topic, payload);
                self.producer.send(&kafka_record)
            }
        }
        .or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "filter", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    cursor: Cursor,

    pub input: MapperInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Debug, Clone, Deserialize)]
pub enum PartitionStrategy {
    ByBlock,
    Random,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub brokers: Vec<String>,
    pub topic: String,
    pub ack_timeout_secs: Option<u64>,
    pub paritioning: Option<PartitionStrategy>,
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
