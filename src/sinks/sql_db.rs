use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

pub struct Worker {
    db: sqlx::Pool<sqlx::Any>,
}

fn hbs_data(point: Point, record: Option<Record>) -> serde_json::Value {
    serde_json::json!({
        "point": match point {
            Point::Origin => serde_json::Value::Null,
            Point::Specific(slot, hash) => serde_json::json!({
                "slot": slot,
                "hash": hex::encode(hash),
            }),
        },
        "record": serde_json::Value::from(record),
    })
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let db = sqlx::AnyPool::connect(&stage.config.connection)
            .await
            .or_retry()?;

        Ok(Self { db })
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

        let template = match unit {
            ChainEvent::Apply(p, r) => {
                let data = hbs_data(p.clone(), Some(r.clone()));
                match r {
                    Record::CborBlock(_) => stage.templates.render("apply_cbor_block", &data),
                    Record::CborTx(_) => stage.templates.render("apply_cbor_tx", &data),
                    _ => stage.templates.render("apply", &data),
                }
            }
            ChainEvent::Undo(p, r) => {
                let data = hbs_data(p.clone(), Some(r.clone()));
                match r {
                    Record::CborBlock(_) => stage.templates.render("undo_cbor_block", &data),
                    Record::CborTx(_) => stage.templates.render("undo_cbor_tx", &data),
                    _ => stage.templates.render("undo", &data),
                }
            }
            ChainEvent::Reset(p) => {
                let data = hbs_data(p.clone(), None);
                stage.templates.render("reset_cbor_block", &data).ok();
                stage.templates.render("reset_cbor_tx", &data)
            }
        };

        let statement = template.or_panic()?;

        let result = sqlx::query(&statement).execute(&self.db).await.or_retry()?;
        debug!(rows = result.rows_affected(), "sql statement executed");

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sql", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    templates: handlebars::Handlebars<'static>,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    /// eg: sqlite::memory:
    pub connection: String,
    pub apply_cbor_block_template: String,
    pub undo_cbor_block_template: String,
    pub apply_cbor_tx_template: String,
    pub undo_cbor_tx_template: String,
    pub reset_cbor_block_template: String,
    pub reset_cbor_tx_template: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        sqlx::any::install_default_drivers();

        let mut templates = handlebars::Handlebars::new();

        templates
            .register_template_string("apply_cbor_block", &self.apply_cbor_block_template)
            .map_err(Error::config)?;

        templates
            .register_template_string("undo_cbor_block", &self.undo_cbor_block_template)
            .map_err(Error::config)?;

        templates
            .register_template_string("apply_cbor_tx", &self.apply_cbor_tx_template)
            .map_err(Error::config)?;

        templates
            .register_template_string("undo_cbor_tx", &self.undo_cbor_tx_template)
            .map_err(Error::config)?;

        templates
            .register_template_string("reset_cbor_block", &self.reset_cbor_block_template)
            .map_err(Error::config)?;

        templates
            .register_template_string("reset_cbor_tx", &self.reset_cbor_tx_template)
            .map_err(Error::config)?;

        let stage = Stage {
            config: self,
            templates,
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
