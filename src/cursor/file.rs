use std::path::PathBuf;

use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tokio::select;

use crate::framework::*;

fn breadcrumbs_to_data(crumbs: &Breadcrumbs) -> Vec<(u64, String)> {
    crumbs
        .points()
        .into_iter()
        .map(|p| match p {
            Point::Origin => None,
            Point::Specific(slot, hash) => Some((slot, hex::encode(hash))),
        })
        .flatten()
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

#[derive(Default)]
pub struct Worker {}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Default::default())
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
                let file = std::fs::File::options()
                    .write(true)
                    .create(true)
                    .append(false)
                    .truncate(true)
                    .open(&stage.path)
                    .or_panic()?;

                let data = breadcrumbs_to_data(&stage.breadcrumbs);
                serde_json::to_writer_pretty(&file, &data).or_panic()?;
            }
        }

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "cursor", unit = "Unit", worker = "Worker")]
pub struct Stage {
    path: std::path::PathBuf,

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
    pub path: Option<PathBuf>,
    pub max_breadcrumbs: Option<usize>,
    pub flush_interval: Option<u64>,
}

impl Config {
    fn define_path(&self) -> Result<PathBuf, Error> {
        let path = self.path.clone();

        let path = match path {
            Some(x) => x,
            None => std::env::current_dir()
                .map_err(Error::custom)?
                .join("cursor.json"),
        };

        Ok(path)
    }

    pub fn initial_load(&self) -> Result<Breadcrumbs, Error> {
        let path = self.define_path()?;

        let max_breadcrumbs = self
            .max_breadcrumbs
            .clone()
            .unwrap_or(DEFAULT_MAX_BREADCRUMBS);

        if path.is_file() {
            let file = std::fs::File::open(&path).map_err(|err| Error::Custom(err.to_string()))?;
            let data: Vec<(u64, String)> = serde_json::from_reader(&file).map_err(Error::custom)?;
            let crumbs = breadcrumbs_from_data(data, max_breadcrumbs)?;

            Ok(crumbs)
        } else {
            Ok(Breadcrumbs::new(max_breadcrumbs))
        }
    }

    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let flush_interval = self
            .flush_interval
            .clone()
            .unwrap_or(DEFAULT_FLUSH_INTERVAL as u64);

        let stage = Stage {
            path: self.define_path()?,
            breadcrumbs: ctx.breadcrumbs.clone(),
            tracked_slot: Default::default(),
            flush_count: Default::default(),
            track: Default::default(),
            flush: gasket::messaging::TimerPort::from_secs(flush_interval),
        };

        Ok(stage)
    }
}
