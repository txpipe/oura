use aws_sdk_s3::Client as S3Client;
use gasket::framework::*;
use gasket::messaging::SendPort;
use serde::Deserialize;

use crate::framework::*;

#[derive(Stage)]
#[stage(name = "source")]
pub struct Stage {
    bucket: String,
    items_per_batch: u32,
    cursor: Cursor,

    retry_policy: gasket::retries::Policy,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

impl gasket::framework::Stage for Stage {
    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy {
            work_retry: self.retry_policy.clone(),
            bootstrap_retry: self.retry_policy.clone(),
            ..Default::default()
        }
    }
}

impl Stage {
    pub fn spawn(self) -> Result<Vec<gasket::runtime::Tether>, Error> {
        let tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![tether])
    }
}

pub struct Worker {
    s3_client: S3Client,
    last_key: String,
}

pub struct KeyBatch {
    keys: Vec<String>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = KeyBatch;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
        let sdk_config = aws_config::load_from_env().await;
        let s3_client = aws_sdk_s3::Client::new(&sdk_config);

        let p = stage
            .cursor
            .latest_known_point()
            .unwrap_or(pallas::network::miniprotocols::Point::Origin);

        let key = match p {
            pallas::network::miniprotocols::Point::Origin => "origin".to_owned(),
            pallas::network::miniprotocols::Point::Specific(slot, _) => format!("{slot}"),
        };

        Ok(Self {
            s3_client,
            last_key: key,
        })
    }

    async fn schedule(
        &mut self,
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let result = self
            .s3_client
            .list_objects_v2()
            .bucket(&stage.bucket)
            .max_keys(stage.items_per_batch as i32)
            .start_after(self.last_key.clone())
            .send()
            .await
            .or_retry()?;

        let keys = result
            .contents
            .unwrap_or_default()
            .into_iter()
            .filter_map(|obj| obj.key)
            .collect::<Vec<_>>();

        Ok(WorkSchedule::Unit(KeyBatch { keys }))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        for key in &unit.keys {
            let object = self
                .s3_client
                .get_object()
                .bucket(&stage.bucket)
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

            stage.output_port.send(event.into()).await.or_panic()?;
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    bucket: String,
    items_per_batch: u32,
    retry_policy: gasket::retries::Policy,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            bucket: self.bucket,
            items_per_batch: self.items_per_batch,
            retry_policy: self.retry_policy,
            cursor: ctx.cursor.clone(),
            output_port: Default::default(),
            ops_count: Default::default(),
        };

        Ok(stage)
    }
}
