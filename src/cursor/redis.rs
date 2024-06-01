use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use r2d2_redis::{
    r2d2::{self, Pool},
    redis::{self, Commands},
    RedisConnectionManager,
};
use serde::Deserialize;
use tokio::select;
use tracing::debug;

use crate::framework::*;

fn breadcrumbs_to_data(crumbs: &Breadcrumbs) -> Vec<(u64, String)> {
    crumbs
        .points()
        .into_iter()
        .filter_map(|p| match p {
            Point::Origin => None,
            Point::Specific(slot, hash) => Some((slot, hex::encode(hash))),
        })
        .collect()
}

fn breadcrumbs_from_data(data: Vec<(u64, String)>, max: usize) -> Result<Breadcrumbs, Error> {
    let points: Vec<_> = data
        .into_iter()
        .map::<Result<_, Error>, _>(|(slot, hash)| {
            let hash = hex::decode(hash).map_err(Error::custom)?;
            Ok(Point::Specific(slot, hash))
        })
        .collect::<Result<_, _>>()?;

    Ok(Breadcrumbs::from_points(points, max))
}

pub enum Unit {
    Track(Point),
    Flush,
}

pub struct Worker {
    pool: Pool<RedisConnectionManager>,
    key: String,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let manager = RedisConnectionManager::new(stage.url.clone()).or_panic()?;
        let pool = r2d2::Pool::builder().build(manager).or_panic()?;

        Ok(Self {
            pool,
            key: stage.key.clone(),
        })
    }

    async fn schedule(&mut self, stage: &mut Stage) -> Result<WorkSchedule<Unit>, WorkerError> {
        select! {
            msg = stage.track.recv() => {
                let msg = msg.or_panic()?;
                Ok(WorkSchedule::Unit(Unit::Track(msg.payload)))
            }
            msg = stage.flush.recv() => {
                msg.or_panic()?;
                Ok(WorkSchedule::Unit(Unit::Flush))
            }
        }
    }

    async fn execute(&mut self, unit: &Unit, stage: &mut Stage) -> Result<(), WorkerError> {
        match unit {
            Unit::Track(x) => stage.breadcrumbs.track(x.clone()),
            Unit::Flush => {
                let data = breadcrumbs_to_data(&stage.breadcrumbs);
                let mut conn = self.pool.get().or_restart()?;

                let data_to_write = serde_json::to_string(&data).or_panic()?;
                conn.set(&self.key, &data_to_write)
                    .map_err(Error::custom)
                    .or_panic()?;
            }
        }

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "cursor", unit = "Unit", worker = "Worker")]
pub struct Stage {
    key: String,
    url: String,

    breadcrumbs: Breadcrumbs,

    pub track: gasket::messaging::InputPort<Point>,

    pub flush: gasket::messaging::TimerPort,

    #[metric]
    tracked_slot: gasket::metrics::Gauge,

    #[metric]
    flush_count: gasket::metrics::Counter,
}

const DEFAULT_MAX_BREADCRUMBS: usize = 10;
const DEFAULT_FLUSH_INTERVAL: usize = 10;

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub key: String,
    pub url: String,
    pub max_breadcrumbs: Option<usize>,
    pub flush_interval: Option<u64>,
}

impl Config {
    pub fn initial_load(&self) -> Result<Breadcrumbs, Error> {
        let client = redis::Client::open(self.url.clone())
            .map_err(|err| Error::Custom(format!("Unable to connect to Redis: {}", err)))?;
        let mut conn = client
            .get_connection()
            .map_err(|err| Error::Custom(format!("Unable to establish connection: {}", err)))?;

        let max_breadcrumbs = self.max_breadcrumbs.unwrap_or(DEFAULT_MAX_BREADCRUMBS);

        let result: redis::RedisResult<Option<String>> = conn.get(&self.key);

        match result {
            Ok(Some(data_as_string)) => {
                debug!("Retrieving cursor information from redis.");
                let data: Vec<(u64, String)> =
                    serde_json::from_str(&data_as_string).map_err(Error::custom)?;
                let crumbs = breadcrumbs_from_data(data, max_breadcrumbs)?;
                Ok(crumbs)
            }
            Ok(None) => {
                debug!("No cursor information found on redis cluster.");
                Ok(Breadcrumbs::new(max_breadcrumbs))
            }
            Err(err) => Err(Error::custom(err)),
        }
    }

    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let flush_interval = self.flush_interval.unwrap_or(DEFAULT_FLUSH_INTERVAL as u64);

        let stage = Stage {
            key: self.key.clone(),
            url: self.url.clone(),
            breadcrumbs: ctx.breadcrumbs.clone(),
            tracked_slot: Default::default(),
            flush_count: Default::default(),
            track: Default::default(),
            flush: gasket::messaging::TimerPort::from_secs(flush_interval),
        };

        Ok(stage)
    }
}
