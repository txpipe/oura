use std::ops::DerefMut;

use gasket::framework::*;
use r2d2_redis::{
    r2d2::{self, Pool},
    redis, RedisConnectionManager,
};
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    pool: Pool<RedisConnectionManager>,
    stream: String,
    maxlen: Option<usize>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let manager = RedisConnectionManager::new(stage.config.url.clone()).or_panic()?;
        let pool = r2d2::Pool::builder().build(manager).or_panic()?;

        let stream = stage
            .config
            .stream_name
            .clone()
            .unwrap_or(String::from("oura-sink"));

        let maxlen = stage.config.stream_max_length;

        Ok(Self {
            pool,
            stream,
            maxlen,
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

        let payload = serde_json::Value::from(record.unwrap()).to_string();

        let mut conn = self.pool.get().or_restart()?;

        let mut command = redis::cmd("XADD");
        command.arg(self.stream.clone());

        if let Some(maxlen) = self.maxlen {
            command.arg("MAXLEN");
            command.arg(maxlen);
        }

        command
            .arg("*")
            .arg(&[point.slot_or_default().to_string(), payload])
            .query(conn.deref_mut())
            .or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-redis", unit = "ChainEvent", worker = "Worker")]
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
pub enum StreamStrategy {
    ByBlock,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub stream_name: Option<String>,
    pub stream_max_length: Option<usize>,
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
