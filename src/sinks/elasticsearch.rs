use elasticsearch::{
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    params::OpType,
    Elasticsearch, IndexParts,
};
use gasket::framework::*;
use serde::{Deserialize, Serialize};

use crate::framework::*;

#[derive(Serialize)]
struct ESRecord {
    #[serde(flatten)]
    event: serde_json::Value,
    #[serde(rename = "@timestamp")]
    timestamp: u64,
}
impl ESRecord {
    pub fn new(record: Record, slot: u64) -> Self {
        Self {
            event: serde_json::Value::from(record),
            timestamp: slot,
        }
    }
}

pub struct Worker {
    client: Elasticsearch,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let url = Url::parse(&stage.config.url).or_panic()?;
        let pool = SingleNodeConnectionPool::new(url);
        let mut transport =
            TransportBuilder::new(pool).cert_validation(CertificateValidation::None);

        if let Some(credentials) = &stage.config.credentials {
            transport = transport.auth(credentials.into());
        }

        let client = Elasticsearch::new(transport.build().or_panic()?);

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

        let slot = point.slot_or_default();
        let slot_str = slot.to_string();

        let mut parts = IndexParts::Index(&stage.config.index);
        if stage.config.idempotency {
            parts = IndexParts::IndexId(&stage.config.index, &slot_str);
        }

        let timestamp = stage.genesis.slot_to_wallclock(slot);
        let payload = ESRecord::new(record.unwrap(), timestamp);

        self.client
            .index(parts)
            .body(payload)
            .op_type(OpType::Create)
            .send()
            .await
            .or_retry()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(slot as i64);
        stage.cursor.send(point.into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-elasticsearch", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    genesis: GenesisValues,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialsConfig {
    Basic { username: String, password: String },
}

impl From<&CredentialsConfig> for Credentials {
    fn from(other: &CredentialsConfig) -> Self {
        match other {
            CredentialsConfig::Basic { username, password } => {
                Credentials::Basic(username.clone(), password.clone())
            }
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub index: String,
    pub credentials: Option<CredentialsConfig>,
    #[serde(default)]
    pub idempotency: bool,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let Chain::Cardano(chain_config) = &ctx.chain;

        let stage = Stage {
            config: self,
            genesis: chain_config.clone().into(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
