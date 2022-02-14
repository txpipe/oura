use super::prelude::*;

pub(crate) fn block_depth_doesnt_skip_numbers(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from((curr.number - prev.number) == 1),
        _ => Outcome::Unknown,
    }
}

pub(crate) fn block_slot_increases(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from(prev.slot < curr.slot),
        _ => Outcome::Unknown,
    }
}

pub(crate) fn event_timestamp_increases(state: &State) -> Outcome {
    match (&state.previous_event, &state.current_event) {
        (Some(prev), Some(curr)) => match (prev.context.timestamp, curr.context.timestamp) {
            (Some(prev), Some(curr)) => Outcome::from(prev <= curr),
            _ => Outcome::Unknown,
        },
        _ => Outcome::Unknown,
    }
}

pub(crate) fn block_previous_hash_matches(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from(curr.previous_hash == prev.hash),
        _ => Outcome::Unknown,
    }
}

pub(crate) fn tx_records_matches_block_count(state: &State) -> Outcome {
    match &state.current_event {
        Some(event) => match &event.data {
            crate::model::EventData::BlockEnd(block) => {
                Outcome::from(block.tx_count == state.tx_count_in_block)
            }
            _ => Outcome::Unknown,
        },
        _ => Outcome::Unknown,
    }
}
