//! A noop filter used as example and placeholder for other filters

use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

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

gasket::stateless_mapper!(Worker, |stage: Stage, unit: ChainEvent| => {
    let out = unit.clone();
    stage.ops_count.inc(1);
    out
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
