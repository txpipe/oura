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

use crate::Error;

pub use crate::sources::PointArg;

pub(crate) trait CursorProvider {
    fn read_cursor(&self) -> Option<PointArg>;
    fn write_cursor(&self, point: PointArg) -> Result<(), Error>;
}

#[derive(Clone)]
struct Cursor {
    point: PointArg,
    reached: Instant,
}

/// A cursor provider that uses the file system as the source for persistence
pub(crate) struct FileProvider {
    cursor: RwLock<Option<Cursor>>,
}

impl FileProvider {
    pub fn initialize() -> Self {
        let maybe_point = match FileProvider::read_from_file() {
            Ok(x) => x,
            Err(err) => {
                log::warn!(
                    "failure loading cursor from file, will continue without it: {}",
                    err
                );
                None
            }
        };

        let cursor = maybe_point.map(|point| Cursor {
            point,
            reached: Instant::now(),
        });

        Self {
            cursor: RwLock::new(cursor),
        }
    }

    fn read_from_file() -> Result<Option<PointArg>, Error> {
        let file = std::fs::read_to_string("foo.txt");

        match file {
            Ok(data) => Ok(Some(data.parse()?)),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(format!("failed loading cursor file: {}", err).into()),
            },
        }
    }

    fn dump_to_file(&self, point: &PointArg) -> Result<(), Error> {
        std::fs::write("foo.txt", point.to_string().as_bytes())?;

        Ok(())
    }
}

impl CursorProvider for FileProvider {
    fn read_cursor(&self) -> Option<PointArg> {
        let guard = self.cursor.read().expect("error prior to acquiring lock");
        guard.as_ref().map(|x| x.point.clone())
    }

    fn write_cursor(&self, point: PointArg) -> Result<(), Error> {
        let mut guard = self.cursor.write().unwrap();

        if let Some(cursor) = guard.as_ref() {
            if cursor.reached.elapsed() > Duration::from_secs(10) {
                self.dump_to_file(&point)?;
            }
        }

        *guard = Some(Cursor {
            reached: Instant::now(),
            point,
        });

        Ok(())
    }
}
