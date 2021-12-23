use elasticsearch::{Elasticsearch, IndexParts};
use log::{debug, error};
use serde::Serialize;
use serde_json::json;
use std::sync::{mpsc::Receiver, Arc};

use crate::framework::{Error, Event};

#[derive(Serialize)]
struct ESRecord {
    #[serde(flatten)]
    event: Event,

    variant: String,

    // we need this field so that our data plays nicely with Elasticsearch "data streams".
    #[serde(rename = "@timestamp")]
    timestamp: Option<u64>,
}

impl From<Event> for ESRecord {
    fn from(event: Event) -> Self {
        let timestamp = event.context.timestamp.map(|x| x * 1000).clone();
        let variant = event.data.to_string();

        ESRecord {
            event,
            timestamp,
            variant,
        }
    }
}

async fn index_event(client: Arc<Elasticsearch>, index: &str, evt: Event) -> Result<(), Error> {
    let record = ESRecord::from(evt);

    let response = client
        .index(IndexParts::Index(&index))
        .body(json!(record))
        .send()
        .await?;

    if response.status_code().is_success() {
        debug!("pushed event to elastic");
    } else {
        error!(
            "error pushing event to elastic: {:?}",
            response.text().await
        );
    }

    Ok(())
}

pub fn writer_loop(
    input: Receiver<Event>,
    client: Elasticsearch,
    index: String,
) -> Result<(), Error> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    loop {
        let index = index.to_owned();
        let evt = input.recv().unwrap();
        let client = client.clone();
        rt.block_on(async move {
            index_event(client, &index, evt).await.unwrap();
        });
    }
}
