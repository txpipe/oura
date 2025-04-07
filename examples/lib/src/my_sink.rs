use std::env;

use gasket::framework::*;
use oura::framework::*;
use tracing::{info, warn};

#[derive(Default, Stage)]
#[stage(name = "my-sink", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

/// Values/State that will be alive on the stage, like connections.
pub struct Worker {
    db: sqlx::sqlite::SqlitePool,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    /// Initialize the worker and its values.
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        let url = env::var("DATABASE_URL").or_panic()?;
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&url)
            .await
            .or_panic()?;

        sqlx::migrate!("src/migrations").run(&db).await.or_panic()?;

        Ok(Self { db })
    }

    /// Wait for inputs in the stage and schedule to be executed.
    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    /// Business execution logic
    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let record = unit.record().cloned();
        let point = unit.point().clone();

        if let Some(record) = record {
            match record {
                Record::ParsedTx(tx) => {
                    let id = hex::encode(tx.hash.clone());
                    let data = serde_json::to_string(&tx).or_panic()?;

                    sqlx::query!(
                        r#"
                            INSERT OR REPLACE INTO tx(
                                id,
	                              data
                            )
                            VALUES ($1, $2);
                        "#,
                        id,
                        data
                    )
                    .execute(&self.db)
                    .await
                    .or_panic()?;

                    info!("new tx persisted");
                }
                _ => warn!("my_sink only supports parsedTx, enable split_block and parse_cbor"),
            }
        }

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}
