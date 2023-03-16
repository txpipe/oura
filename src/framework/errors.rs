use serde::{Deserialize, Serialize};
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

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ErrorAction {
    Skip,
    Warn,
    Default,
}

impl Default for ErrorAction {
    fn default() -> Self {
        ErrorAction::Default
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RuntimePolicy {
    pub parse_errors: Option<ErrorAction>,
    pub any_error: Option<ErrorAction>,
}

#[inline]
fn handle_error<T>(err: Error, action: &Option<ErrorAction>) -> Result<Option<T>, Error> {
    match action {
        Some(ErrorAction::Skip) => Ok(None),
        Some(ErrorAction::Warn) => {
            log::warn!("{}", err);
            Ok(None)
        }
        _ => Err(err),
    }
}

pub trait AppliesPolicy {
    type Value;

    fn apply_policy(self, policy: &RuntimePolicy) -> Result<Option<Self::Value>, Error>;
}

impl<T> AppliesPolicy for Result<T, Error> {
    type Value = T;

    fn apply_policy(self, policy: &RuntimePolicy) -> Result<Option<Self::Value>, Error> {
        match self {
            Ok(t) => Ok(Some(t)),
            Err(err) => {
                // apply generic error policy if we have one
                if matches!(policy.any_error, Some(_)) {
                    return handle_error(err, &policy.any_error);
                }

                // apply specific actions for each type of error
                match &err {
                    Error::Parse(_) => handle_error(err, &policy.parse_errors),
                    _ => Err(err),
                }
            }
        }
    }
}
