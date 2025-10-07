use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::framework::cardano::legacy_v1;
use crate::framework::*;

use self::checks::*;
use self::prelude::*;

mod checks;
mod prelude;

macro_rules! run_check {
    ($config:expr, $state:expr, $func:ident) => {
        let outcome = $func($state);
        let name = stringify!($func);

        if $config.skip_assertions.iter().any(|x| x.eq(&name)) {
            debug!("skipped assertion: {}", name);
        } else {
            match outcome {
                Outcome::Pass => {
                    info!("passed assertion: {}", name);
                }
                Outcome::Fail => {
                    error!("failed assertion: {}", name);
                    dbg!($state);

                    if $config.break_on_failure {
                        panic!("failed assertion in assert sink");
                    }
                }
                Outcome::Unknown => {
                    warn!("unknown assertion outcome: {}", name);
                }
                Outcome::NotApplicable => (),
            };
        }
    };
}

fn reduce_state(current: State, event: legacy_v1::Event) -> State {
    let state = match &event.data {
        legacy_v1::EventData::Block(r) => State {
            previous_block: current.current_block,
            current_block: Some(r.clone()),
            tx_records_since_block: 0,
            ..current
        },
        legacy_v1::EventData::Transaction(_) => State {
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

pub struct Worker {
    state: State,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        let state = State::default();
        Ok(Self { state })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let event = match record.unwrap() {
            Record::Cardano(cardano::Record::OuraV1Event(event)) => Ok(event),
            _ => Err(Error::config(String::from("Only legacy_v1 events"))),
        }
        .or_panic()?;

        self.state = reduce_state(self.state.clone(), event);

        run_check!(&stage.config, &self.state, block_depth_doesnt_skip_numbers);
        run_check!(&stage.config, &self.state, block_slot_increases);
        run_check!(&stage.config, &self.state, block_previous_hash_matches);
        run_check!(&stage.config, &self.state, event_timestamp_increases);
        run_check!(&stage.config, &self.state, tx_records_matches_block_count);
        run_check!(&stage.config, &self.state, tx_has_input_and_output);

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.send(point.clone().into()).await.or_panic()?;

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "sink-assert", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,

    pub input: MapperInputPort,
    pub cursor: SinkCursorPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub break_on_failure: bool,

    #[serde(default = "Vec::new")]
    pub skip_assertions: Vec<String>,
}

impl Config {
    pub fn bootstrapper(self, _: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: Default::default(),
        };

        Ok(stage)
    }
}
