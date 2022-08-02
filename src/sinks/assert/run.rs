use std::sync::Arc;

use crate::{
    model::{Event, EventData},
    pipelining::StageReceiver,
    utils::Utils,
    Error,
};

use super::checks::*;
use super::prelude::*;
use super::Config;

macro_rules! run_check {
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
                        panic!("failed assertion in assert sink");
                    }
                }
                Outcome::Unknown => {
                    log::warn!("unknown assertion outcome: {}", name);
                }
                Outcome::NotApplicable => (),
            };
        }
    };
}

fn reduce_state(current: State, event: Event) -> State {
    let state = match &event.data {
        EventData::Block(r) => State {
            previous_block: current.current_block,
            current_block: Some(r.clone()),
            tx_records_since_block: 0,
            ..current
        },
        EventData::Transaction(_) => State {
            tx_records_since_block: current.tx_records_since_block + 1,
            ..current
        },
        _ => current,
    };

    State {
        previous_event: state.current_event,
        current_event: Some(event),
        ..state
    }
}

pub fn assertion_loop(
    input: StageReceiver,
    config: Config,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    let mut state = State::default();

    for event in input.iter() {
        log::info!("starting assertions for event: {:?}", event.fingerprint);

        state = reduce_state(state, event);

        run_check!(config, &state, block_depth_doesnt_skip_numbers);
        run_check!(config, &state, block_slot_increases);
        run_check!(config, &state, block_previous_hash_matches);
        run_check!(config, &state, event_timestamp_increases);
        run_check!(config, &state, tx_records_matches_block_count);
        run_check!(config, &state, tx_has_input_and_output);

        if let Some(event) = &state.current_event {
            // notify pipeline about the progress
            utils.track_sink_progress(event);
        }
    }

    Ok(())
}
