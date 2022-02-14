use crate::model::{BlockRecord, Event};

#[derive(Default, Debug)]
pub(crate) struct State {
    pub current_event: Option<Event>,
    pub previous_event: Option<Event>,
    pub current_block: Option<BlockRecord>,
    pub previous_block: Option<BlockRecord>,
    pub tx_records_since_block: usize,
}

pub(crate) enum Outcome {
    Pass,
    Fail,
    NotApplicable,
    Unknown,
}

impl From<bool> for Outcome {
    fn from(other: bool) -> Self {
        match other {
            true => Outcome::Pass,
            false => Outcome::Fail,
        }
    }
}
