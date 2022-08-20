use elasticsearch::{params::OpType, Elasticsearch, IndexParts};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::{
    model::Event,
    pipelining::StageReceiver,
    utils::{retry, Utils},
};

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
    parts: IndexParts<'_>,
    event: Event,
) -> Result<(), String> {
    let req_body = json!(ESRecord::from(event));

    let response = client
        .index(parts)
        .body(req_body)
        .op_type(OpType::Create)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    match response.status_code() {
        StatusCode::CONFLICT => {
            log::warn!("skiping event since it already exists");
            Ok(())
        }
        x if x.is_success() => {
            log::debug!("pushed event to elastic");
            Ok(())
        }
        x => {
            if let Ok(body) = response.text().await {
                log::debug!("error response from ES: {}", body);
            }

            Err(format!("response with status code {}", x))
        }
    }
}

async fn index_event_with_id<'b>(
    client: Arc<Elasticsearch>,
    index: &'_ str,
    event: Event,
) -> Result<(), String> {
    let fingerprint = event.fingerprint.clone();

    let parts = match &fingerprint {
        Some(id) => IndexParts::IndexId(index, id),
        _ => {
            log::warn!("trying to index with idempotency but no event fingerprint available");
            IndexParts::Index(index)
        }
    };

    index_event(client, parts, event).await
}

async fn index_event_without_id<'b>(
    client: Arc<Elasticsearch>,
    index: &'_ str,
    event: Event,
) -> Result<(), String> {
    let parts = IndexParts::Index(index);
    index_event(client, parts, event).await
}

fn execute_fallible_request(
    client: &Arc<Elasticsearch>,
    event: &Event,
    idempotency: bool,
    index: &str,
    rt: &Runtime,
) -> Result<(), String> {
    let client = client.clone();
    let event = event.clone();

    rt.block_on(async move {
        match idempotency {
            true => index_event_with_id(client, index, event).await,
            false => index_event_without_id(client, index, event).await,
        }
    })
}

pub fn writer_loop(
    input: StageReceiver,
    client: Elasticsearch,
    index: String,
    idempotency: bool,
    retry_policy: retry::Policy,
    utils: Arc<Utils>,
) -> Result<(), String> {
    let client = Arc::new(client);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .map_err(|x| x.to_string())?;

    for event in input.iter() {
        let index = index.to_owned();
        let client = client.clone();

        let result = retry::retry_operation(
            || execute_fallible_request(&client, &event, idempotency, &index, &rt),
            &retry_policy,
        );

        match result {
            Ok(_) => {
                // notify progress to the pipeline
                utils.track_sink_progress(&event);
            }
            Err(err) => {
                format!("error indexing record in Elasticsearch: {}", err);
                return Err(err);
            }
        }
    }

    Ok(())
}
