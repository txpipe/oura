//! A noop filter used as example and placeholder for other filters

use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use serde::Deserialize;
use std::borrow::Cow;

use pallas::ledger::traverse as trv;

use crate::framework::*;

type CborBlock<'a> = Cow<'a, [u8]>;
type CborTx<'a> = Cow<'a, [u8]>;

fn map_block_to_tx(cbor: CborBlock) -> Result<Vec<CborTx>, WorkerError> {
    let block = trv::MultiEraBlock::decode(cbor.as_ref()).or_panic()?;

    let txs: Vec<_> = block
        .txs()
        .iter()
        .map(|tx| tx.encode())
        .map(Cow::Owned)
        .collect();

    Ok(txs)
}

#[derive(Default)]
pub struct Stage {
    ops_count: gasket::metrics::Counter,
    input: FilterInputPort,
    output: FilterOutputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "filter"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
    }
}

gasket::stateless_flatmapper!(Worker, |stage: Stage, unit: ChainEvent| => {
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

    stage.ops_count.inc(1);

    output
});

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }
}

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        Ok(Stage::default())
    }
}
