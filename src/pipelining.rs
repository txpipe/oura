use std::thread::JoinHandle;

use crate::{model::Event, Error};

pub type StageReceiver = std::sync::mpsc::Receiver<Event>;

pub type StageSender = std::sync::mpsc::SyncSender<Event>;

/// The amount of events an inter-stage channel can buffer before blocking
///
/// If a filter or sink has a consumption rate lower than the rate of event
/// generations from a source, the pending events will buffer in a queue
/// provided by the corresponding mpsc channel implementation. This constant
/// defines the max amount of events that the buffer queue can hold. Once
/// reached, the previous stages in the pipeline will start blockin on 'send'.
///
/// This value has a direct effect on the amount of memory consumed by the
/// process. The higher the buffer, the higher potential memory consumption.
///
/// This value has a direct effect on performance. To allow _pipelining_
/// benefits, stages should be allowed certain degree of flexibility to deal
/// with resource constrains (such as network or cpu). The lower the buffer, the
/// lower degree of flexibility.
const DEFAULT_INTER_STAGE_BUFFER_SIZE: usize = 1000;

pub type StageChannel = (StageSender, StageReceiver);

/// Centralizes the implementation details of inter-stage channel creation
///
/// Concrete channel implementation is subject to change. We're still exploring
/// sync vs unbounded and threaded vs event-loop. Until we have a long-term
/// strategy, it makes sense to have a single place in the codebase that can be
/// used to change from one implementation to the other without incurring on
/// heavy refactoring throughout several files.
///
/// Sometimes centralization is not such a bad thing :)
pub fn new_inter_stage_channel(buffer_size: Option<usize>) -> StageChannel {
    std::sync::mpsc::sync_channel(buffer_size.unwrap_or(DEFAULT_INTER_STAGE_BUFFER_SIZE))
}

pub type PartialBootstrapResult = Result<(JoinHandle<()>, StageReceiver), Error>;

pub type BootstrapResult = Result<JoinHandle<()>, Error>;

pub trait SourceProvider {
    fn bootstrap(&self) -> PartialBootstrapResult;
}

pub trait FilterProvider {
    fn bootstrap(&self, input: StageReceiver) -> PartialBootstrapResult;
}

pub trait SinkProvider {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult;
}
