use crossterm::style::{Color, Print, Stylize};
use crossterm::ExecutableCommand;
use gasket::error::AsWorkError;
use pallas::network::upstream::cursor::Cursor;
use std::io::Stdout;

use crate::framework::*;

use super::format::*;
use super::throttle::Throttle;

pub struct Worker {
    pub(crate) msg_count: gasket::metrics::Counter,
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
            .with_counter("msg_count", &self.msg_count)
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

        let (point, line) = match msg.payload {
            ChainEvent::Apply(point, record) => {
                let line = LogLine::new_apply(&record, width, &self.adahandle_policy);
                (point, line)
            }
            ChainEvent::Undo(point, record) => {
                let line = LogLine::new_undo(&record, width, &self.adahandle_policy);
                (point, line)
            }
            ChainEvent::Reset(point) => {
                let line = LogLine::new_reset(point.clone());
                (point, line)
            }
        };

        self.stdout.execute(Print(line)).or_panic()?;

        self.cursor.add_breadcrumb(point);

        self.throttle.wait_turn();

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
