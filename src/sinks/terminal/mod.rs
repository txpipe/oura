use crossterm::style::{Color, Print, Stylize};
use crossterm::ExecutableCommand;
use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
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
impl gasket::framework::Worker for Worker {
    type Unit = ChainEvent;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
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
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
        let width = self.compute_terminal_width(stage.config.wrap.unwrap_or_default());

        let point = unit.point().clone();

        let line = match unit {
            ChainEvent::Apply(_, record) => {
                LogLine::new_apply(&record, width, &stage.config.adahandle_policy)
            }
            ChainEvent::Undo(_, record) => {
                LogLine::new_undo(&record, width, &stage.config.adahandle_policy)
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

pub struct Stage {
    config: Config,
    latest_block: gasket::metrics::Gauge,
    ops_count: gasket::metrics::Counter,
    cursor: Cursor,
    input: MapperInputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "sink"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
        registry.track_gauge("latest_block", &self.latest_block);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }
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
