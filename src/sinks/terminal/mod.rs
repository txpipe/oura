use crossterm::style::{Color, Print, Stylize};
use crossterm::ExecutableCommand;
use gasket::framework::*;
use serde::Deserialize;
use std::io::{stdout, Stdout};

use crate::framework::*;

mod format;
mod throttle;

use format::*;
use throttle::Throttle;

pub struct Worker {
    stdout: Stdout,
    throttle: Throttle,
}

impl Worker {
    fn compute_terminal_width(&self, wrap: bool) -> Option<usize> {
        if !wrap {
            return None;
        }

        if let Ok((x, _y)) = crossterm::terminal::size() {
            return Some(x as usize);
        }

        None
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let mut stdout = stdout();

        stdout
            .execute(Print(
                "Oura terminal output started, waiting for chain data\n".with(Color::DarkGrey),
            ))
            .or_panic()?;

        let worker = Self {
            stdout,
            throttle: stage.config.throttle_min_span_millis.into(),
        };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let width = self.compute_terminal_width(stage.config.wrap.unwrap_or_default());

        let point = unit.point().clone();

        let line = match unit {
            ChainEvent::Apply(_, record) => {
                LogLine::new_apply(record, width, &stage.config.adahandle_policy)
            }
            ChainEvent::Undo(_, record) => {
                LogLine::new_undo(record, width, &stage.config.adahandle_policy)
            }
            ChainEvent::Reset(point) => LogLine::new_reset(point.clone()),
        };

        self.throttle.wait_turn();
        self.stdout.execute(Print(line)).or_panic()?;

        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point);

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "filter", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    cursor: Cursor,

    pub input: MapperInputPort,

    #[metric]
    latest_block: gasket::metrics::Gauge,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub throttle_min_span_millis: Option<u64>,
    pub wrap: Option<bool>,
    pub adahandle_policy: Option<String>,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
            cursor: ctx.cursor.clone(),
        };

        Ok(stage)
    }
}
