use std::sync::mpsc::Receiver;

use elasticsearch::{
    auth::Credentials as ESCredentials,
    cert::CertificateValidation,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch,
};

use serde_derive::Deserialize;

use crate::framework::{BootstrapResult, Event, SinkConfig};

use super::run::writer_loop;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialsConfig {
    Basic { username: String, password: String },
}

impl Into<ESCredentials> for &CredentialsConfig {
    fn into(self) -> ESCredentials {
        match self {
            CredentialsConfig::Basic { username, password } => {
                ESCredentials::Basic(username.clone(), password.clone())
            }
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    url: String,
    index: String,
    credentials: Option<CredentialsConfig>,
}

impl SinkConfig for Config {
    fn bootstrap(&self, input: Receiver<Event>) -> BootstrapResult {
        let pool = SingleNodeConnectionPool::new(Url::parse(&self.url)?);
        let mut transport =
            TransportBuilder::new(pool).cert_validation(CertificateValidation::None);

        if let Some(creds) = &self.credentials {
            transport = transport.auth(creds.into());
        };

        let client = Elasticsearch::new(transport.build()?);

        let index = self.index.clone();
        let handle = std::thread::spawn(move || {
            writer_loop(input, client, index).expect("writer loop failed")
        });

        Ok(handle)
    }
}
