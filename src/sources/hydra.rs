use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;

use pallas::network::miniprotocols::Point;

use gasket::framework::*;
use tracing::{debug, error, info, warn};

use futures_util::StreamExt;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::framework::*;

#[derive(PartialEq, Debug, Clone)]
pub struct HydraMessage {
    pub seq: u64,
    pub head_id: Option<Vec<u8>>,
    pub payload: HydraMessagePayload,
    pub raw_json: Value,
}

impl HydraMessage {
    fn head_id_or_default(&self) -> Vec<u8> {
        let dummy_hash = vec![0u8; 28];
        self.head_id.clone().unwrap_or(dummy_hash)
    }

    /// As a first implementation, we'll treat the msg seq number
    /// as a slot number, and the head id as a block hash.
    ///
    /// This means all points on the same chain will share the same block hash, but hopefully
    /// this shouldn't matter.
    fn pseudo_point(&self) -> Point {
        Point::Specific(self.seq, self.head_id_or_default())
    }
}

impl<'de> Deserialize<'de> for HydraMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: Value = Deserialize::deserialize(deserializer)?;

        let seq = map
            .get("seq")
            .ok_or_else(|| de::Error::missing_field("seq"))?
            .as_u64()
            .ok_or_else(|| de::Error::custom("seq should be a u64"))?;

        let head_id_str: Option<&str> = map
            .get("headId")
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| de::Error::custom("headId should be a string"))
            })
            .transpose()?;

        let head_id = head_id_str
            .map(|s| {
                hex::decode(s)
                    .map_err(|_e| serde::de::Error::custom(format!("Expected hex-encoded headId")))
            })
            .transpose()?;

        let payload = HydraMessagePayload::deserialize(&map).map_err(de::Error::custom)?;
        let raw_json = map;

        Ok(HydraMessage {
            seq,
            payload,
            raw_json,
            head_id,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "tag", rename_all = "PascalCase")]
pub enum HydraMessagePayload {
    #[serde(deserialize_with = "deserialize_tx_valid")]
    TxValid { tx: Vec<u8> },

    #[serde(other)]
    Other,
}

fn deserialize_tx_valid<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TxValidJson {
        transaction: TxCborJson,
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TxCborJson {
        cbor_hex: String,
    }

    let msg = TxValidJson::deserialize(deserializer)?;
    let cbor = hex::decode(msg.transaction.cbor_hex)
        .map_err(|_e| serde::de::Error::custom(format!("Expected hex-encoded cbor")))?;

    Ok(cbor)
}

type HydraConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Stage)]
#[stage(name = "source", unit = "Message", worker = "Worker")]
pub struct Stage {
    config: Config,

    intersect: IntersectConfig,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    current_slot: gasket::metrics::Gauge,
}

pub struct Worker {
    socket: HydraConnection,
    intersect: WorkerIntersect,
}

/// Worker state for finding the right intersection point
#[derive(Debug, Clone)]
pub enum WorkerIntersect {
    SkipUntil(u64, Vec<u8>), // Possibility of Point::Origin is excluded
    ProcessMessages,
}

impl Worker {
    async fn process(&mut self, stage: &mut Stage, msg: HydraMessage) -> Result<(), WorkerError> {
        let point = msg.pseudo_point();
        match &self.intersect {
            WorkerIntersect::SkipUntil(slot, hash) => {
                let target = Point::Specific(slot.clone(), hash.clone());
                debug!(
                    "Skipping message {} before (or at) requested intersection {}",
                    point.slot_or_default(), target.slot_or_default()
                );

                if target == point {
                    self.intersect = WorkerIntersect::ProcessMessages;
                } else if point.slot_or_default() >= target.slot_or_default() {
                    warn!(
                        "Skipping message from wrong hydra chain (intersection point hash mismatch) {:?} {:?}",
                        target, point
                    );
                }
                Ok(())
            }
            WorkerIntersect::ProcessMessages => self.process_message(stage, msg).await,
        }
    }

    /// Helper only to be called by `process`
    async fn process_message(
        &mut self,
        stage: &mut Stage,
        next: HydraMessage,
    ) -> Result<(), WorkerError> {
        let point = next.pseudo_point();

        // First, apply raw json messages regardless of message type
        let json_evt = ChainEvent::Apply(point.clone(), Record::GenericJson(next.raw_json.clone()));
        stage.output.send(json_evt.into()).await.or_panic()?;
        stage.ops_count.inc(1);

        // Apply CborTx events for any txs
        match next.payload {
            HydraMessagePayload::TxValid { tx } => {
                let evt = ChainEvent::Apply(point.clone(), Record::CborTx(tx));
                stage.output.send(evt.into()).await.or_panic()?;
                stage.ops_count.inc(1);

                stage.current_slot.set(point.slot_or_default() as i64);
                stage.ops_count.inc(1);
            }
            _ => (),
        };
        Ok(())
    }
}

fn intersect_from_config(intersect: &IntersectConfig) -> WorkerIntersect {
    match intersect {
        IntersectConfig::Origin => {
            info!("starting from Origin");
            WorkerIntersect::ProcessMessages
        }
        IntersectConfig::Tip => {
            panic!("intersecting tip not currently supported with hydra as source")
        }
        IntersectConfig::Point(slot, hash_str) => {
            info!("intersecting specific point");
            let hash = hex::decode(hash_str).expect("valid hex hash");
            WorkerIntersect::SkipUntil(slot.clone(), hash)
        }
        IntersectConfig::Breadcrumbs(_) => {
            panic!("intersecting breadcrumbs not currently supported with hydra as source")
        }
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting to hydra WebSocket");

        let url = &stage.config.ws_url;
        let (socket, _) = connect_async(url).await.expect("Can't connect");
        let worker = Self {
            socket,
            intersect: intersect_from_config(&stage.intersect),
        };

        Ok(worker)
    }

    async fn schedule(&mut self, _stage: &mut Stage) -> Result<WorkSchedule<Message>, WorkerError> {
        let next_msg = self.socket.next().await.transpose().or_restart()?;

        Ok(match next_msg {
            Some(message) => WorkSchedule::Unit(message),
            None => WorkSchedule::Idle,
        })
    }

    async fn execute(&mut self, message: &Message, stage: &mut Stage) -> Result<(), WorkerError> {
        match message {
            Message::Text(text) => {
                debug!("Hydra message: {}", text);
                match serde_json::from_str::<HydraMessage>(text) {
                    Ok(hydra_message) => {
                        self.process(stage, hydra_message).await
                    }
                    Err(err) => {
                        error!("Failed to parse Hydra message: {}", err);
                        Ok(())
                    }
                }
            }
            _ => Ok(()),
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub ws_url: String,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            intersect: ctx.intersect.clone(),
            output: Default::default(),
            ops_count: Default::default(),
            current_slot: Default::default(),
        };

        Ok(stage)
    }
}
