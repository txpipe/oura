use elasticsearch::{
    auth::Credentials as ESCredentials,
    cert::CertificateValidation,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch,
};

use serde::Deserialize;

use crate::{
    pipelining::{BootstrapResult, SinkProvider, StageReceiver},
    utils::{retry, WithUtils},
};

use super::run::writer_loop;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialsConfig {
    Basic { username: String, password: String },
}

impl From<&CredentialsConfig> for ESCredentials {
    fn from(other: &CredentialsConfig) -> Self {
        match other {
            CredentialsConfig::Basic { username, password } => {
                ESCredentials::Basic(username.clone(), password.clone())
            }
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub index: String,
    pub credentials: Option<CredentialsConfig>,

    #[serde(default)]
    pub idempotency: bool,

    pub retry_policy: Option<retry::Policy>,
}

impl SinkProvider for WithUtils<Config> {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let pool = SingleNodeConnectionPool::new(Url::parse(&self.inner.url)?);
        let mut transport =
            TransportBuilder::new(pool).cert_validation(CertificateValidation::None);

        if let Some(creds) = &self.inner.credentials {
            transport = transport.auth(creds.into());
        };

        let client = Elasticsearch::new(transport.build()?);

        let index = self.inner.index.clone();
        let idempotency = self.inner.idempotency;
        let retry_policy = self.inner.retry_policy.unwrap_or_default();
        let utils = self.utils.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, client, index, idempotency, retry_policy, utils)
                .expect("writer loop failed")
        });

        Ok(handle)
    }
}
