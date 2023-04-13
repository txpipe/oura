use gasket::error::AsWorkError;
use pallas::network::miniprotocols::Point;

use crate::framework::*;

pub struct Worker {
    pub(crate) client: reqwest::Client,
    pub(crate) url: String,
    pub(crate) ops_count: gasket::metrics::Counter,
    pub(crate) latest_block: gasket::metrics::Gauge,
    pub(crate) cursor: Cursor,
    pub(crate) input: MapperInputPort,
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = ChainEvent;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .with_gauge("latest_block", &self.latest_block)
            .build()
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;
        Ok(gasket::runtime::WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let body = serde_json::Value::from(record.unwrap());

        let point_header = match &point {
            Point::Origin => String::from("origin"),
            Point::Specific(a, b) => format!("{a},{}", hex::encode(b)),
        };

        let request = self
            .client
            .post(&self.url)
            .header("x-oura-chainsync-action", "undo")
            .header("x-oura-chainsync-point", point_header)
            .json(&body)
            .build()
            .or_panic()?;

        self.client
            .execute(request)
            .await
            .and_then(|res| res.error_for_status())
            .or_retry()?;

        self.ops_count.inc(1);

        self.latest_block.set(point.slot_or_default() as i64);
        self.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}
