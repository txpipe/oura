use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config error")]
    Config(String),

    #[error("{0}")]
    Custom(String),

    #[error("parse error {0}")]
    Parse(String),
}

impl Error {
    pub fn config(err: impl ToString) -> Self {
        Self::Config(err.to_string())
    }

    pub fn custom(err: impl ToString) -> Self {
        Self::Custom(err.to_string())
    }

    pub fn parse(error: impl ToString) -> Self {
        Self::Parse(error.to_string())
    }
}
