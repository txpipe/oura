///! An utility to keep track of the progress of the pipeline as a whole
use prometheus_exporter::prometheus::{register_counter, register_int_gauge, Counter, IntGauge};

use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::{
    model::{Event, EventData},
    Error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub binding: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Clone)]
pub struct Tip {
    pub block: u64,
    pub slot: u64,
}

#[derive(Default, Merge, Clone)]
pub(crate) struct ChainState {
    pub tip: Option<Tip>,
}

pub(crate) struct Provider {
    pub chain_tip: IntGauge,
    pub rollback_count: Counter,
    pub source_current_slot: IntGauge,
    pub source_current_height: IntGauge,
    pub source_event_count: Counter,
    pub sink_current_slot: IntGauge,
    pub sink_event_count: Counter,
}

impl Provider {
    pub(crate) fn initialize(config: &Config) -> Result<Self, Error> {
        let binding = config
            .binding
            .as_deref()
            .unwrap_or("0.0.0.0:9186")
            .parse()?;

        let mut builder = prometheus_exporter::Builder::new(binding);

        if let Some(endpoint) = &config.endpoint {
            builder.with_endpoint(endpoint)?;
        }

        builder.start()?;

        let provider = Provider {
            chain_tip: register_int_gauge!(
                "chain_tip",
                "the last detected tip of the chain (height)"
            )?,
            rollback_count: register_counter!(
                "rollback_count",
                "number of rollback events occurred"
            )?,
            source_current_slot: register_int_gauge!(
                "source_current_slot",
                "last slot processed by the source of the pipeline"
            )?,
            source_current_height: register_int_gauge!(
                "source_current_height",
                "last height (block #) processed by the source of the pipeline"
            )?,
            source_event_count: register_counter!(
                "source_event_count",
                "number of events processed by the source of the pipeline"
            )?,
            sink_current_slot: register_int_gauge!(
                "sink_current_slot",
                "last slot processed by the sink of the pipeline"
            )?,
            sink_event_count: register_counter!(
                "sink_event_count",
                "number of events processed by the sink of the pipeline"
            )?,
        };

        Ok(provider)
    }

    pub(crate) fn on_chain_tip(&self, tip: u64) {
        self.chain_tip.set(tip as i64);
    }

    pub(crate) fn on_source_event(&self, event: &Event) {
        self.source_event_count.inc();

        if let Some(slot) = &event.context.slot {
            self.source_current_slot.set(*slot as i64);
        }

        if let Some(block) = &event.context.block_number {
            self.source_current_height.set(*block as i64);
        }

        if matches!(event.data, EventData::RollBack { .. }) {
            self.rollback_count.inc();
        }
    }

    pub(crate) fn on_sink_event(&self, event: &Event) {
        self.sink_event_count.inc();

        if let Some(slot) = &event.context.slot {
            self.sink_current_slot.set(*slot as i64);
        }
    }
}
