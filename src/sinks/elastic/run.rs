use elasticsearch::{Elasticsearch, IndexParts};
use log::{debug, error, warn};
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
        let timestamp = event.context.timestamp.map(|x| x * 1000);
        let variant = event.data.to_string();

        ESRecord {
            event,
            timestamp,
            variant,
        }
    }
}

#[inline]
async fn index_event<'b>(
    client: Arc<Elasticsearch>,
    parts: IndexParts<'b>,
    event: Event,
) -> Result<(), Error> {
    let req_body = json!(ESRecord::from(event));

    let response = client.index(parts).body(req_body).send().await?;

    if response.status_code().is_success() {
        debug!("pushed event to elastic");
        Ok(())
    } else {
        let msg = format!(
            "error pushing event to elastic: {:?}",
            response.text().await
        );

        Err(msg.into())
    }
}

async fn index_event_with_id<'b>(
    client: Arc<Elasticsearch>,
    index: &'b str,
    event: Event,
) -> Result<(), Error> {
    let fingerprint = event.fingerprint.clone();

    let parts = match &fingerprint {
        Some(id) => IndexParts::IndexId(index, id),
        _ => {
            warn!("trying to index with idempotency but no event fingerprint available");
            IndexParts::Index(index)
        }
    };

    index_event(client, parts, event).await
}

async fn index_event_without_id<'b>(
    client: Arc<Elasticsearch>,
    index: &'b str,
    event: Event,
) -> Result<(), Error> {
    let parts = IndexParts::Index(index);
    index_event(client, parts, event).await
}

pub fn writer_loop(
    input: Receiver<Event>,
    client: Elasticsearch,
    index: String,
    idempotency: bool,
) -> Result<(), Error> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    loop {
        let index = index.to_owned();
        let event = input.recv().unwrap();
        let client = client.clone();
        rt.block_on(async move {
            let result = match idempotency {
                true => index_event_with_id(client, &index, event).await,
                false => index_event_without_id(client, &index, event).await,
            };

            if let Err(err) = result {
                warn!("error indexing record in Elasticsearch: {}", err);
            }
        });
    }
}
