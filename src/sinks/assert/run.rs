use std::sync::Arc;

use crate::{
    model::{BlockRecord, Event, EventData},
    pipelining::StageReceiver,
    utils::Utils,
    Error,
};

use super::Config;

#[derive(Default, Debug)]
struct State {
    current_event: Option<Event>,
    previous_event: Option<Event>,
    current_block: Option<BlockRecord>,
    previous_block: Option<BlockRecord>,
}

enum Outcome {
    Pass,
    Fail,
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

macro_rules! execute_assertion {
    ($config:expr, $state:expr, $func:ident) => {
        let outcome = $func($state);
        let name = stringify!($func);

        if $config.skip_assertions.iter().any(|x| x.eq(&name)) {
            log::debug!("skipped assertion: {}", name);
        } else {
            match outcome {
                Outcome::Pass => {
                    log::info!("passed assertion: {}", name);
                }
                Outcome::Fail => {
                    log::error!("failed assertion: {}", name);
                    dbg!($state);

                    if $config.break_on_failure {
                        panic!();
                    }
                }
                Outcome::Unknown => {
                    log::warn!("unknown assertion outcome: {}", name);
                }
            };
        }
    };
}

fn block_depth_doesnt_skip_numbers(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from((curr.number - prev.number) == 1),
        _ => Outcome::Unknown,
    }
}

fn block_slot_increases(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from(prev.slot < curr.slot),
        _ => Outcome::Unknown,
    }
}

fn event_timestamp_increases(state: &State) -> Outcome {
    match (&state.previous_event, &state.current_event) {
        (Some(prev), Some(curr)) => match (prev.context.timestamp, curr.context.timestamp) {
            (Some(prev), Some(curr)) => Outcome::from(prev <= curr),
            _ => Outcome::Unknown,
        },
        _ => Outcome::Unknown,
    }
}

fn block_previous_hash_matches(state: &State) -> Outcome {
    match (&state.previous_block, &state.current_block) {
        (Some(prev), Some(curr)) => Outcome::from(curr.previous_hash == prev.hash),
        _ => Outcome::Unknown,
    }
}

/*
fn cbor_decoding_is_isomorphic(state: &State) -> bool {
    match state.latest_event {
        Some(event) => match event.data {
            EventData::Block(block) => {

            }
        }
    }
}
 */
fn reduce_state(current: State, event: Event) -> State {
    let state = match &event.data {
        EventData::Block(r) => State {
            previous_block: current.current_block,
            current_block: Some(r.clone()),
            ..current
        },
        _ => current,
    };

    let state = State {
        previous_event: state.current_event,
        current_event: Some(event),
        ..state
    };

    state
}

pub fn assertion_loop(
    input: StageReceiver,
    config: Config,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let mut state = State::default();

    loop {
        let event = input.recv()?;

        // notify pipeline about the progress
        utils.track_sink_progress(&event);

        log::info!("starting assertions for event: {:?}", event.fingerprint);

        state = reduce_state(state, event);

        execute_assertion!(&config, &state, block_depth_doesnt_skip_numbers);
        execute_assertion!(&config, &state, block_slot_increases);
        execute_assertion!(&config, &state, block_previous_hash_matches);
        execute_assertion!(&config, &state, event_timestamp_increases);
    }
}
