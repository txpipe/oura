use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::framework::*;

use super::common::web::{build_headers_map, APP_USER_AGENT};

fn build_request(
    client: &reqwest::Client,
    url: &str,
    unit: &ChainEvent,
) -> Result<reqwest::Request, reqwest::Error> {
    let point = unit.point();

    let point_header = match point {
        Point::Origin => String::from("origin"),
        Point::Specific(a, b) => format!("{a},{}", hex::encode(b)),
    };

    let (action, body) = match unit {
        ChainEvent::Apply(_, record) => ("apply", serde_json::Value::from(record.clone())),
        ChainEvent::Undo(_, record) => ("undo", serde_json::Value::from(record.clone())),
        ChainEvent::Reset(_) => (
            "reset",
            serde_json::json!({
                "event": "reset",
                "point": match point {
                    Point::Origin => serde_json::Value::from("origin"),
                    Point::Specific(slot, hash) => serde_json::json!({
                        "slot": slot,
                        "hash": hex::encode(hash),
                    }),
                },
            }),
        ),
    };

    client
        .post(url)
        .header("x-oura-chainsync-action", action)
        .header("x-oura-chainsync-point", point_header)
        .json(&body)
        .build()
}

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
            .danger_accept_invalid_certs(stage.config.allow_invalid_certs.unwrap_or(false))
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

        let request = build_request(&self.client, &stage.config.url, unit).or_panic()?;

        self.client
            .execute(request)
            .await
            .and_then(|res| res.error_for_status())
            .or_retry()?;

        stage.ops_count.inc(1);

        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-webhook", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn point() -> Point {
        Point::Specific(42, vec![0xab, 0xcd])
    }

    fn request_for(unit: ChainEvent) -> (String, String, serde_json::Value) {
        let request =
            build_request(&reqwest::Client::new(), "http://localhost/hook", &unit).unwrap();
        let header = |name: &str| request.headers()[name].to_str().unwrap().to_owned();
        let body = request.body().unwrap().as_bytes().unwrap();

        (
            header("x-oura-chainsync-action"),
            header("x-oura-chainsync-point"),
            serde_json::from_slice(body).unwrap(),
        )
    }

    #[test]
    fn apply_events_post_their_record() {
        let record = Record::CborBlock(vec![1, 2]);
        let (action, point_header, body) = request_for(ChainEvent::Apply(point(), record.clone()));

        assert_eq!(action, "apply");
        assert_eq!(point_header, "42,abcd");
        assert_eq!(body, serde_json::Value::from(record));
    }

    #[test]
    fn undo_events_are_not_reported_as_apply() {
        let record = Record::CborBlock(vec![1, 2]);
        let (action, _, body) = request_for(ChainEvent::Undo(point(), record.clone()));

        assert_eq!(action, "undo");
        assert_eq!(body, serde_json::Value::from(record));
    }

    #[test]
    fn reset_events_post_the_point() {
        let (action, point_header, body) = request_for(ChainEvent::Reset(point()));

        assert_eq!(action, "reset");
        assert_eq!(point_header, "42,abcd");
        assert_eq!(
            body,
            serde_json::json!({"event": "reset", "point": {"slot": 42, "hash": "abcd"}})
        );
    }

    #[test]
    fn a_reset_to_origin_posts_origin() {
        let (action, point_header, body) = request_for(ChainEvent::Reset(Point::Origin));

        assert_eq!(action, "reset");
        assert_eq!(point_header, "origin");
        assert_eq!(body["point"], "origin");
    }
}
