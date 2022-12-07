use std::sync::Arc;

use crate::{
    model::{Era, Event, EventContext, EventData},
    pipelining::StageSender,
    utils::{time::TimeProvider, Utils},
};

use merge::Merge;
use serde::Deserialize;

use crate::Error;

#[deprecated]
pub use crate::utils::ChainWellKnownInfo;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub include_block_end_events: bool,

    #[serde(default)]
    pub include_transaction_details: bool,

    #[serde(default)]
    pub include_transaction_end_events: bool,

    #[serde(default)]
    pub include_invalid_transaction_details: bool,

    #[serde(default)]
    pub include_block_details: bool,

    #[serde(default)]
    pub include_block_cbor: bool,

    #[serde(default)]
    pub include_byron_ebb: bool,
}

#[derive(Clone)]
pub struct EventWriter {
    context: EventContext,
    output: StageSender,
    pub(crate) config: Config,
    pub(crate) utils: Arc<Utils>,
}

impl EventWriter {
    pub fn new(output: StageSender, utils: Arc<Utils>, config: Config) -> Self {
        EventWriter {
            context: EventContext::default(),
            output,
            utils,
            config,
        }
    }

    #[allow(unused)]
    pub fn standalone(
        output: StageSender,
        well_known: Option<ChainWellKnownInfo>,
        config: Config,
    ) -> Self {
        let utils = Arc::new(Utils::new(well_known.unwrap_or_default()));

        Self::new(output, utils, config)
    }

    pub fn append(&self, data: EventData) -> Result<(), Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        self.utils.track_source_progress(&evt);

        self.output
            .send(evt)
            .expect("error sending event through output stage, pipeline must have crashed.");

        Ok(())
    }

    pub fn append_from<T>(&self, source: T) -> Result<(), Error>
    where
        T: Into<EventData>,
    {
        self.append(source.into())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            output: self.output.clone(),
            utils: self.utils.clone(),
            config: self.config.clone(),
        }
    }

    pub fn compute_timestamp(&self, slot: u64) -> Option<u64> {
        match &self.utils.time {
            Some(provider) => provider.slot_to_wallclock(slot).into(),
            _ => None,
        }
    }
}

impl From<pallas::ledger::traverse::Era> for Era {
    fn from(other: pallas::ledger::traverse::Era) -> Self {
        match other {
            pallas::ledger::traverse::Era::Byron => Era::Byron,
            pallas::ledger::traverse::Era::Shelley => Era::Shelley,
            pallas::ledger::traverse::Era::Allegra => Era::Allegra,
            pallas::ledger::traverse::Era::Mary => Era::Mary,
            pallas::ledger::traverse::Era::Alonzo => Era::Alonzo,
            pallas::ledger::traverse::Era::Babbage => Era::Babbage,
            _ => Era::Unknown,
        }
    }
}
