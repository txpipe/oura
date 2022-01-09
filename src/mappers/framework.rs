use crate::framework::{ChainWellKnownInfo, Event, EventContext, EventData, StageSender};
use merge::Merge;
use serde_derive::Deserialize;

pub type Error = Box<dyn std::error::Error>;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct MapperConfig {
    #[serde(default)]
    pub include_transaction_details: bool,
}

#[derive(Clone, Debug)]
pub struct EventWriter {
    context: EventContext,
    output: StageSender,
    chain_info: Option<ChainWellKnownInfo>,
    pub mapping_config: MapperConfig,
}

impl EventWriter {
    pub fn new(
        output: StageSender,
        chain_info: Option<ChainWellKnownInfo>,
        mapping_config: MapperConfig,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            chain_info,
            mapping_config,
        }
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        self.output.send(evt)?;

        Ok(())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
            chain_info: self.chain_info.clone(),
            mapping_config: self.mapping_config.clone(),
        }
    }

    pub fn compute_timestamp(&self, slot: u64) -> Option<u64> {
        self.chain_info
            .as_ref()
            .map(|info| info.shelley_known_time + (slot - info.shelley_known_slot))
    }
}

/// IoC for mapping raw data into Oura events
pub trait EventMapper<S> {
    fn map_events(&self, source: &S, writer: &EventWriter) -> Result<(), Error>;
}
