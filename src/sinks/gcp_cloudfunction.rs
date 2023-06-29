use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufReader, Read},
    time::Duration,
};

use gasket::framework::*;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};

use crate::framework::*;

use super::common::web::{build_headers_map, APP_USER_AGENT};

#[derive(Serialize)]
struct Claims {
    pub iss: String,
    pub aud: String,
    pub target_audience: String,
    pub iat: u64,
    pub exp: u64,
}
impl Claims {
    pub fn new(audience: &String, credentials: &Credentials) -> Self {
        let iat = jsonwebtoken::get_current_timestamp();
        let exp = iat + 60;
        Self {
            iss: credentials.client_email.clone(),
            aud: credentials.token_uri.clone(),
            target_audience: audience.clone(),
            iat,
            exp,
        }
    }
}

#[derive(Deserialize)]
struct AuthResponse {
    pub id_token: String,
}

struct Credentials {
    pub client_email: String,
    pub token_uri: String,
    pub private_key: String,
}
impl TryFrom<serde_json::Value> for Credentials {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let client_email = value.get("client_email");
        let token_uri = value.get("token_uri");
        let private_key = value.get("private_key");

        if client_email.is_none() || token_uri.is_none() || private_key.is_none() {
            return Err(Error::Config(String::from("Invalid credentials file")));
        }

        let client_email = client_email.unwrap().as_str().unwrap().to_string();
        let token_uri = token_uri.unwrap().as_str().unwrap().to_string();
        let private_key = private_key.unwrap().as_str().unwrap().to_string();

        Ok(Self {
            client_email,
            token_uri,
            private_key,
        })
    }
}

pub struct GCPAuth {
    client: reqwest::Client,
    credentials: Credentials,
    audience: String,
    token: Option<String>,
}
impl GCPAuth {
    pub fn try_new(audience: String) -> Result<Self, Error> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .build()?;

        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        let value: serde_json::Value = serde_json::from_str(&contents)?;

        let credentials: Credentials = value.try_into()?;

        Ok(Self {
            client,
            credentials,
            audience,
            token: None,
        })
    }

    pub async fn get_token(&mut self) -> Result<String, Error> {
        if self.token_is_valid() {
            return Ok(self.token.as_ref().unwrap().clone());
        }

        self.refresh_token().await?;

        Ok(self.token.as_ref().unwrap().clone())
    }

    async fn refresh_token(&mut self) -> Result<(), Error> {
        let header = Header::new(Algorithm::RS256);
        let claims = Claims::new(&self.audience, &self.credentials);
        let key = EncodingKey::from_rsa_pem(self.credentials.private_key.as_bytes())?;
        let token = jsonwebtoken::encode(&header, &claims, &key)?;

        let form = multipart::Form::new()
            .text("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer")
            .text("assertion", token);

        let response = self
            .client
            .post(&self.credentials.token_uri)
            .multipart(form)
            .send()
            .await
            .and_then(|res| res.error_for_status())?;

        let auth_response = response.json::<AuthResponse>().await?;
        self.token = Some(auth_response.id_token);

        Ok(())
    }

    fn token_is_valid(&self) -> bool {
        if self.token.is_none() {
            return false;
        }

        let key = DecodingKey::from_secret(&[]);
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();

        jsonwebtoken::decode::<serde_json::Value>(self.token.as_ref().unwrap(), &key, &validation)
            .is_ok()
    }
}

pub struct Worker {
    client: Client,
    gcp_auth: Option<GCPAuth>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let headers = build_headers_map(None, stage.config.headers.as_ref()).or_panic()?;

        let client = reqwest::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_millis(stage.config.timeout.unwrap_or(30000)))
            .build()
            .or_panic()?;

        let mut worker = Self {
            client,
            gcp_auth: None,
        };

        if stage.config.authentication {
            worker.gcp_auth = Some(GCPAuth::try_new(stage.config.url.clone()).or_panic()?);
        }

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let payload = serde_json::Value::from(record.unwrap());

        let mut request_builder = self.client.post(&stage.config.url).json(&payload);

        if let Some(gcp_auth) = self.gcp_auth.as_mut() {
            let token = gcp_auth.get_token().await.or_restart()?;
            let authorization = format!("Bearer {token}");
            let headers = build_headers_map(Some(&authorization), None).or_panic()?;
            request_builder = request_builder.headers(headers);
        }

        let request = request_builder.build().or_panic()?;

        self.client
            .execute(request)
            .await
            .and_then(|res| res.error_for_status())
            .or_restart()?;

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "filter", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    cursor: Cursor,

    pub input: MapperInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub timeout: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub authentication: bool,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(stage)
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        Error::Config(value.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Config(value.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Config(value.to_string())
    }
}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Custom(value.to_string())
    }
}
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Error::Custom(value.to_string())
    }
}
