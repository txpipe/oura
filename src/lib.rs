use thiserror::Error;

pub mod filters;
pub mod framework;
pub mod mapper;
pub mod sinks;
pub mod sources;
pub mod utils;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error {0}")]
    Parse(String),
}

impl Error {
    pub fn parse(error: impl ToString) -> Self {
        Error::Parse(error)
    }
}
