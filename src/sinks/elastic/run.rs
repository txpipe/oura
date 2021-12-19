use elasticsearch::{Elasticsearch, IndexParts};
use log::{debug, error, info};
use std::sync::{mpsc::Receiver, Arc};

use crate::framework::{Error, Event};

async fn index_event(client: Arc<Elasticsearch>, index: &str, evt: Event) -> Result<(), Error> {
    let json = serde_json::to_value(evt)?;

    let response = client
        .index(IndexParts::Index(&index))
        .body(json)
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
