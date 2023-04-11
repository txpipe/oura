use std::io::Write;

use file_rotate::suffix::AppendTimestamp;
use file_rotate::FileRotate;
use gasket::error::AsWorkError;
use serde_json::json;
use serde_json::Value as JsonValue;

use crate::framework::*;

pub struct Worker {
    pub(crate) ops_count: gasket::metrics::Counter,
    pub(crate) latest_block: gasket::metrics::Gauge,
    pub(crate) cursor: Cursor,
    pub(crate) writer: FileRotate<AppendTimestamp>,
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
        let (point, json) = match unit {
            ChainEvent::Apply(point, record) => {
                let json = json!({ "event": "apply", "record": JsonValue::from(record.clone()) });
                (point, json)
            }
            ChainEvent::Undo(point, record) => {
                let json = json!({ "event": "undo", "record": JsonValue::from(record.clone()) });
                (point, json)
            }
            ChainEvent::Reset(point) => {
                let json_point = match &point {
                    pallas::network::miniprotocols::Point::Origin => JsonValue::from("origin"),
                    pallas::network::miniprotocols::Point::Specific(slot, hash) => {
                        json!({ "slot": slot, "hash": hex::encode(hash)})
                    }
                };

                let json = json!({ "event": "reset", "point": json_point });
                (point, json)
            }
        };

        self.writer
            .write_all(json.to_string().as_bytes())
            .and_then(|_| self.writer.write_all(b"\n"))
            .or_retry()?;

        self.ops_count.inc(1);

        self.latest_block.set(point.slot_or_default() as i64);
        self.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}
