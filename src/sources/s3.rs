use aws_sdk_s3::Client as S3Client;
use gasket::{error::AsWorkError, messaging::SendPort};
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    s3_client: Option<S3Client>,
    bucket: String,
    items_per_batch: u32,
    output_port: SourceOutputPort,
    ops_count: gasket::metrics::Counter,
}

pub struct KeyBatch {
    keys: Vec<String>,
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = KeyBatch;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    async fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let sdk_config = aws_config::load_from_env().await;
        self.s3_client = Some(aws_sdk_s3::Client::new(&sdk_config));

        Ok(())
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let result = self
            .s3_client
            .as_ref()
            .unwrap()
            .list_objects_v2()
            .bucket(&self.bucket)
            .max_keys(self.items_per_batch as i32)
            .send()
            .await
            .or_retry()?;

        let keys = result
            .contents
            .unwrap_or_default()
            .into_iter()
            .filter_map(|obj| obj.key)
            .collect::<Vec<_>>();

        Ok(gasket::runtime::WorkSchedule::Unit(KeyBatch { keys }))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        for key in &unit.keys {
            let object = self
                .s3_client
                .as_ref()
                .unwrap()
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
                .or_retry()?;

            let metadata = object
                .metadata
                .ok_or("S3 object is missing metadata")
                .or_panic()?;
            let slot = metadata
                .get("slot")
                .ok_or("S3 object is missing block slot")
                .or_panic()?;
            let hash = metadata
                .get("hash")
                .ok_or("S3 object is missing block hash")
                .or_panic()?;

            let point = pallas::network::miniprotocols::Point::Specific(
                slot.parse().or_panic()?,
                hex::decode(hash).or_panic()?,
            );

            let body = object.body.collect().await.or_retry()?;

            let event = ChainEvent::Apply(point, Record::CborBlock(body.into_bytes().to_vec()));

            self.output_port.send(event.into()).await?;
        }

        Ok(())
    }
}

pub struct Bootstrapper(Worker, gasket::retries::Policy);

impl Bootstrapper {
    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.0.output_port.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<gasket::runtime::Tether>, Error> {
        let tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy {
                work_retry: self.1.clone(),
                bootstrap_retry: self.1,
                ..Default::default()
            },
            Some("source"),
        );

        Ok(vec![tether])
    }
}

#[derive(Deserialize)]
pub struct Config {
    bucket: String,
    items_per_batch: u32,
    retry_policy: gasket::retries::Policy,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker {
            s3_client: None,
            bucket: self.bucket,
            items_per_batch: self.items_per_batch,
            output_port: Default::default(),
            ops_count: Default::default(),
        };

        Ok(Bootstrapper(worker, self.retry_policy))
    }
}
