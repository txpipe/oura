//! An utility to keep a cursor of the already processed blocks
//!
//! This is a helper to maintain stateful cursors of the blocks already
//! processed by a pipeline. A source should use this utility to check the
//! initial point from where it should start reading. A sink should use this
//! utility to persist the position once a block has been processed.

use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use serde::Deserialize;

use crate::Error;

pub use crate::sources::PointArg;

pub(crate) trait CanStore {
    fn read_cursor(&self) -> Result<PointArg, Error>;
    fn write_cursor(&self, point: PointArg) -> Result<(), Error>;
}

/// Configuration for the file-based storage implementation
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    path: String,
}

/// A cursor provider that uses the file system as the source for persistence
pub(crate) struct FileStorage(FileConfig);

// TODO: over-engineering a little bit here, leaving room for other
// types of cursor persistence (probably Redis)
enum Storage {
    File(FileStorage),
}

impl CanStore for Storage {
    fn read_cursor(&self) -> Result<PointArg, Error> {
        match self {
            Storage::File(x) => x.read_cursor(),
        }
    }

    fn write_cursor(&self, point: PointArg) -> Result<(), Error> {
        match self {
            Storage::File(x) => x.write_cursor(point),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    File(FileConfig),
}

#[derive(Clone)]
enum State {
    Unknown,
    Invalid,
    AtPoint { point: PointArg, reached: Instant },
}

pub struct Provider {
    storage: Storage,
    state: RwLock<State>,
}

impl Provider {
    fn new(config: Config) -> Self {
        Self {
            state: RwLock::new(State::Unknown),
            storage: match config {
                Config::File(x) => Storage::File(FileStorage(x)),
            },
        }
    }

    pub fn initialize(config: Config) -> Self {
        let new = Provider::new(config);
        new.load_cursor();

        new
    }

    fn load_cursor(&self) {
        let mut guard = self.state.write().expect("error prior to acquiring lock");

        let maybe_point = self.storage.read_cursor();

        if let Err(error) = &maybe_point {
            log::warn!("failure reading cursor from storage: {}", error);
        }

        let state = match maybe_point {
            Ok(point) => State::AtPoint {
                point,
                reached: Instant::now(),
            },
            Err(_) => State::Invalid,
        };

        *guard = state;
    }

    pub fn get_cursor(&self) -> Option<PointArg> {
        let guard = self.state.read().expect("error prior to acquiring lock");

        match &*guard {
            State::AtPoint { point, .. } => Some(point.clone()),
            _ => None,
        }
    }

    pub fn set_cursor(&self, point: PointArg) -> Result<(), Error> {
        let mut guard = self.state.write().unwrap();

        let should_update = match &*guard {
            State::AtPoint { reached, .. } => reached.elapsed() > Duration::from_secs(10),
            _ => true,
        };

        if should_update {
            self.storage.write_cursor(point.clone())?;

            *guard = State::AtPoint {
                reached: Instant::now(),
                point,
            };
        }

        Ok(())
    }
}

impl CanStore for FileStorage {
    fn read_cursor(&self) -> Result<PointArg, Error> {
        let file = std::fs::read_to_string(&self.0.path)?;
        file.parse()
    }

    fn write_cursor(&self, point: PointArg) -> Result<(), Error> {
        std::fs::write(&self.0.path, point.to_string().as_bytes())?;

        Ok(())
    }
}
