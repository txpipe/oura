use std::{collections::HashMap, time::Duration};

use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

use super::common::web::{build_headers_map, APP_USER_AGENT};

pub struct Worker {
    client: reqwest::Client,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let headers = build_headers_map(
            stage.config.authorization.as_ref(),
            stage.config.headers.as_ref(),
        )
        .or_panic()?;

        let client = reqwest::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_millis(stage.config.timeout.unwrap_or(30000)))
            .build()
            .or_panic()?;

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

        let payload = serde_json::Value::from(record.unwrap());

        let request = self
            .client
            .post(&stage.config.url)
            .json(&payload)
            .build()
            .or_panic()?;

        self.client
            .execute(request)
            .await
            .and_then(|res| res.error_for_status())
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

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub timeout: Option<u64>,
    pub authorization: Option<String>,
    pub headers: Option<HashMap<String, String>>,
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
