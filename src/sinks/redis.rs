use gasket::framework::*;
use serde::Deserialize;

use crate::framework::*;

pub struct Worker {
    client: redis::Client,
    stream: String,
    maxlen: Option<usize>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let client = redis::Client::open(stage.config.url.as_str()).or_retry()?;

        let stream = stage
            .config
            .stream_name
            .clone()
            .unwrap_or(String::from("oura-sink"));

        let maxlen = stage.config.stream_max_length;

        Ok(Self {
            client,
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

        let mut conn = self.client.get_connection().or_restart()?;

        let mut command = redis::cmd("XADD");
        command.arg(self.stream.clone());

        if let Some(maxlen) = self.maxlen {
            command.arg("MAXLEN");
            command.arg(maxlen);
        }

        let _: () = command
            .arg("*")
            .arg(&[point.slot_or_default().to_string(), payload])
            .query(&mut conn)
            .or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-redis", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

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
