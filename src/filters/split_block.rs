//! A noop filter used as example and placeholder for other filters

use gasket::{error::AsWorkError, messaging::*, runtime::Tether};
use serde::Deserialize;
use std::borrow::Cow;

use pallas::ledger::traverse as trv;

use crate::framework::*;

#[derive(Default)]
struct Worker {
    ops_count: gasket::metrics::Counter,
    input: FilterInputPort,
    output: FilterOutputPort,
}

type CborBlock<'a> = Cow<'a, [u8]>;
type CborTx<'a> = Cow<'a, [u8]>;

fn map_block_to_tx(cbor: CborBlock) -> Result<Vec<CborTx>, gasket::error::Error> {
    let block = trv::MultiEraBlock::decode(cbor.as_ref()).or_panic()?;

    let txs: Vec<_> = block
        .txs()
        .iter()
        .map(|tx| tx.encode())
        .map(Cow::Owned)
        .collect();

    Ok(txs)
}

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = ChainEvent;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;

        Ok(gasket::runtime::WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        let output = unit.clone().try_map_record_to_many(|r| match r {
            Record::CborBlock(cbor) => {
                let out = map_block_to_tx(Cow::Borrowed(&cbor))?
                    .into_iter()
                    .map(|tx| Record::CborTx(tx.into()))
                    .collect();

                Ok(out)
            }
            x => Ok(vec![x]),
        })?;

        for evt in output {
            self.output.send(evt.into()).await?;
        }

        self.ops_count.inc(1);

        Ok(())
    }
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.0.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("filter"),
        );

        Ok(vec![worker_tether])
    }
}

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Bootstrapper, Error> {
        let worker = Worker::default();

        Ok(Bootstrapper(worker))
    }
}
