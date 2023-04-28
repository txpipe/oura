use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use pallas::network::miniprotocols::Point;
use reqwest::header;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::framework::*;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn build_headers_map(
    authorization: Option<&String>,
    extra: Option<&HashMap<String, String>>,
) -> Result<header::HeaderMap, Error> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::try_from("application/json").map_err(Error::config)?,
    );

    if let Some(auth_value) = &authorization {
        let auth_value = header::HeaderValue::try_from(*auth_value).map_err(Error::config)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    if let Some(custom) = &extra {
        for (name, value) in custom.iter() {
            let name = header::HeaderName::try_from(name).map_err(Error::config)?;
            let value = header::HeaderValue::try_from(value).map_err(Error::config)?;
            headers.insert(name, value);
        }
    }

    Ok(headers)
}

pub struct Worker {
    client: reqwest::Client,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = ChainEvent;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
        let headers = build_headers_map(
            stage.config.authorization.as_ref(),
            stage.config.headers.as_ref(),
        )
        .or_panic()?;

        let client = reqwest::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .danger_accept_invalid_certs(stage.config.allow_invalid_certs.unwrap_or(false))
            .timeout(Duration::from_millis(stage.config.timeout.unwrap_or(30000)))
            .build()
            .or_panic()?;

        Ok(Self { client })
    }

    async fn schedule(
        &mut self,
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let body = serde_json::Value::from(record.unwrap());

        let point_header = match &point {
            Point::Origin => String::from("origin"),
            Point::Specific(a, b) => format!("{a},{}", hex::encode(b)),
        };

        let request = self
            .client
            .post(&stage.config.url)
            .header("x-oura-chainsync-action", "apply")
            .header("x-oura-chainsync-point", point_header)
            .json(&body)
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

pub struct Stage {
    config: Config,
    ops_count: gasket::metrics::Counter,
    latest_block: gasket::metrics::Gauge,
    cursor: Cursor,
    input: MapperInputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "sink"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy {
            work_retry: self.config.retries.clone().unwrap_or_default(),
            ..Default::default()
        }
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
        registry.track_gauge("latest_block", &self.latest_block);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }
}

#[derive(Default, Deserialize)]
pub struct Config {
    pub url: String,
    pub authorization: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout: Option<u64>,

    /// Accept invalid TLS certificates
    ///
    /// DANGER Will Robinson! Set this flag to skip TLS verification. Main
    /// use-case for this flag is to allow self-signed certificates. Beware that
    /// other invalid properties will be omitted too, such as expiration date.
    pub allow_invalid_certs: Option<bool>,

    pub retries: Option<gasket::retries::Policy>,
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
