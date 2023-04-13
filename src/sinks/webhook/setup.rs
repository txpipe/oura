use gasket::{messaging::*, runtime::Tether};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::framework::*;

use super::run::Worker;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Bootstrapper(Worker, gasket::retries::Policy);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy {
                work_retry: self.1,
                ..Default::default()
            },
            Some("sink"),
        );

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

    pub retry_policy: Option<gasket::retries::Policy>,
}

pub fn build_headers_map(
    authorization: Option<&String>,
    extra: Option<&HashMap<String, String>>,
) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::try_from("application/json").map_err(Error::config)?,
    );

    if let Some(auth_value) = &authorization {
        let auth_value = HeaderValue::try_from(*auth_value).map_err(Error::config)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    if let Some(custom) = &extra {
        for (name, value) in custom.iter() {
            let name = HeaderName::try_from(name).map_err(Error::config)?;
            let value = HeaderValue::try_from(value).map_err(Error::config)?;
            headers.insert(name, value);
        }
    }

    Ok(headers)
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker {
            client: reqwest::ClientBuilder::new()
                .user_agent(APP_USER_AGENT)
                .default_headers(build_headers_map(
                    self.authorization.as_ref(),
                    self.headers.as_ref(),
                )?)
                .danger_accept_invalid_certs(self.allow_invalid_certs.unwrap_or(false))
                .timeout(Duration::from_millis(self.timeout.unwrap_or(30000)))
                .build()
                .map_err(Error::config)?,
            url: self.url,
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(Bootstrapper(worker, self.retry_policy.unwrap_or_default()))
    }
}
