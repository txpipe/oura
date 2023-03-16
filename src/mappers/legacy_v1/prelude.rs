use crate::framework::errors::RuntimePolicy;
use crate::framework::legacy_v1::*;
use crate::framework::*;

use merge::Merge;
use pallas::ledger::traverse::wellknown::GenesisValues;
use pallas::network::miniprotocols::Point;

use super::Config;

#[derive(Clone)]
pub struct EventWriter<'a> {
    context: EventContext,
    point: Point,
    output: MapperOutputPort,
    pub(crate) config: &'a Config,
    pub(crate) genesis: &'a GenesisValues,
    pub(crate) error_policy: &'a RuntimePolicy,
}

impl<'a> EventWriter<'a> {
    pub fn new(
        point: Point,
        output: MapperOutputPort,
        config: &'a Config,
        genesis: &'a GenesisValues,
        error_policy: &'a RuntimePolicy,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            point,
            output,
            config,
            genesis,
            error_policy,
        }
    }

    pub fn append(&mut self, data: EventData) -> Result<(), gasket::error::Error> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        let msg = ChainEvent::Apply(self.point.clone(), Record::OuraV1Event(evt)).into();
        self.output.send(msg)?;

        Ok(())
    }

    pub fn append_from<T>(&mut self, source: T) -> Result<(), gasket::error::Error>
    where
        T: Into<EventData>,
    {
        self.append(source.into())
    }

    pub fn child_writer(&self, mut extra_context: EventContext) -> EventWriter {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            point: self.point.clone(),
            output: self.output.clone(),
            config: self.config,
            genesis: self.genesis,
            error_policy: self.error_policy,
        }
    }
}
