use std::io::Write;

use file_rotate::suffix::AppendTimestamp;
use file_rotate::FileRotate;
use gasket::error::AsWorkError;
use pallas::network::upstream::cursor::Cursor;
use serde_json::json;
use serde_json::Value as JsonValue;

use crate::framework::*;

pub struct Worker {
    pub(crate) ops_count: gasket::metrics::Counter,
    pub(crate) cursor: Cursor,
    pub(crate) writer: FileRotate<AppendTimestamp>,
    pub(crate) input: MapperInputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        let (point, json) = match msg.payload {
            ChainEvent::Apply(point, record) => {
                let json = json!({ "event": "apply", "record": JsonValue::from(record) });
                (point, json)
            }
            ChainEvent::Undo(point, record) => {
                let json = json!({ "event": "undo", "record": JsonValue::from(record) });
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

        self.cursor.add_breadcrumb(point);

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
