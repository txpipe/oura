use crate::framework::legacy_v1::*;
use crate::framework::*;

use gasket::framework::WorkerError;
use merge::Merge;
use pallas::ledger::traverse::wellknown::GenesisValues;
use pallas::network::miniprotocols::Point;

use super::Config;

pub struct EventWriter<'a> {
    context: EventContext,
    point: Point,
    output: &'a MapperOutputPort,
    pub(crate) config: &'a Config,
    pub(crate) genesis: &'a GenesisValues,
    buffer: &'a mut Vec<ChainEvent>,
}

impl<'a> EventWriter<'a> {
    pub fn new(
        point: Point,
        output: &'a MapperOutputPort,
        config: &'a Config,
        genesis: &'a GenesisValues,
        buffer: &'a mut Vec<ChainEvent>,
    ) -> Self {
        EventWriter {
            context: EventContext::default(),
            point,
            output,
            config,
            genesis,
            buffer,
        }
    }

    pub fn append(&mut self, data: EventData) -> Result<(), WorkerError> {
        let evt = Event {
            context: self.context.clone(),
            data,
            fingerprint: None,
        };

        let msg = ChainEvent::Apply(self.point.clone(), Record::OuraV1Event(evt));
        self.buffer.push(msg);

        Ok(())
    }

    pub fn append_from<T>(&mut self, source: T) -> Result<(), WorkerError>
    where
        T: Into<EventData>,
    {
        self.append(source.into())
    }

    pub fn child_writer(&mut self, mut extra_context: EventContext) -> EventWriter<'_> {
        extra_context.merge(self.context.clone());

        EventWriter {
            context: extra_context,
            point: self.point.clone(),
            output: self.output,
            config: self.config,
            genesis: self.genesis,
            buffer: self.buffer,
        }
    }
}
