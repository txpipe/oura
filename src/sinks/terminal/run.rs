use crossterm::style::{Color, Print, Stylize};
use crossterm::ExecutableCommand;
use gasket::error::AsWorkError;
use std::io::Stdout;

use crate::framework::*;

use super::format::*;
use super::throttle::Throttle;

pub struct Worker {
    pub(crate) ops_count: gasket::metrics::Counter,
    pub(crate) latest_block: gasket::metrics::Gauge,
    pub(crate) stdout: Stdout,
    pub(crate) throttle: Throttle,
    pub(crate) wrap: bool,
    pub(crate) adahandle_policy: Option<String>,
    pub(crate) cursor: Cursor,
    pub(crate) input: MapperInputPort,
}

impl Worker {
    fn compute_terminal_width(&self) -> Option<usize> {
        if !self.wrap {
            return None;
        }

        if let Ok((x, _y)) = crossterm::terminal::size() {
            return Some(x as usize);
        }

        None
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .with_gauge("latest_block", &self.latest_block)
            .build()
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        self.stdout
            .execute(Print(
                "Oura terminal output started, waiting for chain data\n".with(Color::DarkGrey),
            ))
            .or_panic()?;

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        let width = self.compute_terminal_width();

        let point = msg.payload.point().clone();

        let line = match msg.payload {
            ChainEvent::Apply(_, record) => {
                LogLine::new_apply(&record, width, &self.adahandle_policy)
            }
            ChainEvent::Undo(_, record) => {
                LogLine::new_undo(&record, width, &self.adahandle_policy)
            }
            ChainEvent::Reset(point) => LogLine::new_reset(point.clone()),
        };

        self.stdout.execute(Print(line)).or_panic()?;

        self.latest_block.set(point.slot_or_default() as i64);
        self.cursor.add_breadcrumb(point);

        self.throttle.wait_turn();

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
