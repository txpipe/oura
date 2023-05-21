use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use gasket::{metrics::Reading, runtime::Tether};
use lazy_static::lazy_static;
use tracing::{debug, error, warn};

#[derive(clap::ValueEnum, Clone, Default)]
pub enum Mode {
    /// shows progress as a plain sequence of logs
    #[default]
    Plain,
    /// shows aggregated progress and metrics
    Tui,
}

struct TuiConsole {
    chainsync_progress: indicatif::ProgressBar,
    fetched_blocks: indicatif::ProgressBar,
    plexer_ops_count: indicatif::ProgressBar,
    filter_ops_count: indicatif::ProgressBar,
    mapper_ops_count: indicatif::ProgressBar,
    sink_ops_count: indicatif::ProgressBar,
}

impl TuiConsole {
    fn build_counter_spinner(
        name: &str,
        container: &indicatif::MultiProgress,
    ) -> indicatif::ProgressBar {
        container.add(
            indicatif::ProgressBar::new_spinner().with_style(
                indicatif::ProgressStyle::default_spinner()
                    .template(&format!(
                        "{{spinner}} {name:<20} {{msg:<20}} {{pos:>8}} | {{per_sec}}"
                    ))
                    .unwrap(),
            ),
        )
    }

    fn new() -> Self {
        let container = indicatif::MultiProgress::new();

        Self {
            chainsync_progress: container.add(
                indicatif::ProgressBar::new(0).with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("chainsync progress: {bar} {pos}/{len} eta: {eta}\n{msg}")
                        .unwrap(),
                ),
            ),
            fetched_blocks: Self::build_counter_spinner("fetched blocks", &container),
            plexer_ops_count: Self::build_counter_spinner("plexer ops", &container),
            filter_ops_count: Self::build_counter_spinner("filter ops", &container),
            mapper_ops_count: Self::build_counter_spinner("mapper ops", &container),
            sink_ops_count: Self::build_counter_spinner("sink ops", &container),
        }
    }

    fn refresh<'a>(&self, tethers: impl Iterator<Item = &'a Tether>) {
        for tether in tethers {
            let state = match tether.check_state() {
                gasket::runtime::TetherState::Dropped => "dropped!",
                gasket::runtime::TetherState::Blocked(_) => "blocked!",
                gasket::runtime::TetherState::Alive(x) => match x {
                    gasket::runtime::StagePhase::Bootstrap => "bootstrapping...",
                    gasket::runtime::StagePhase::Working => "working...",
                    gasket::runtime::StagePhase::Teardown => "tearing down...",
                    gasket::runtime::StagePhase::Ended => "ended",
                },
            };

            match tether.read_metrics() {
                Ok(readings) => {
                    for (key, value) in readings {
                        match (tether.name(), key, value) {
                            (_, "chain_tip", Reading::Gauge(x)) => {
                                self.chainsync_progress.set_length(x as u64);
                            }
                            (_, "latest_block", Reading::Gauge(x)) => {
                                self.chainsync_progress.set_position(x as u64);
                            }
                            (_, "fetched_blocks", Reading::Count(x)) => {
                                self.fetched_blocks.set_position(x);
                                self.fetched_blocks.set_message(state);
                            }
                            ("plexer", "ops_count", Reading::Count(x)) => {
                                self.plexer_ops_count.set_position(x);
                                self.plexer_ops_count.set_message(state);
                            }
                            ("filter", "ops_count", Reading::Count(x)) => {
                                self.filter_ops_count.set_position(x);
                                self.filter_ops_count.set_message(state);
                            }
                            ("mapper", "ops_count", Reading::Count(x)) => {
                                self.mapper_ops_count.set_position(x);
                                self.mapper_ops_count.set_message(state);
                            }
                            ("sink", "ops_count", Reading::Count(x)) => {
                                self.sink_ops_count.set_position(x);
                                self.sink_ops_count.set_message(state);
                            }
                            _ => (),
                        }
                    }
                }
                Err(err) => {
                    println!("couldn't read metrics");
                    dbg!(err);
                }
            }
        }
    }
}

struct PlainConsole {
    last_report: Mutex<Instant>,
}

impl PlainConsole {
    fn new() -> Self {
        Self {
            last_report: Mutex::new(Instant::now()),
        }
    }

    fn refresh<'a>(&self, tethers: impl Iterator<Item = &'a Tether>) {
        let mut last_report = self.last_report.lock().unwrap();

        if last_report.elapsed() <= Duration::from_secs(10) {
            return;
        }

        for tether in tethers {
            match tether.check_state() {
                gasket::runtime::TetherState::Dropped => {
                    error!("[{}] stage tether has been dropped", tether.name());
                }
                gasket::runtime::TetherState::Blocked(_) => {
                    warn!(
                        "[{}] stage tehter is blocked or not reporting state",
                        tether.name()
                    );
                }
                gasket::runtime::TetherState::Alive(state) => {
                    debug!("[{}] stage is alive with state: {:?}", tether.name(), state);
                    match tether.read_metrics() {
                        Ok(readings) => {
                            for (key, value) in readings {
                                debug!("[{}] metric `{}` = {:?}", tether.name(), key, value);
                            }
                        }
                        Err(err) => {
                            error!("[{}] error reading metrics: {}", tether.name(), err)
                        }
                    }
                }
            }
        }

        *last_report = Instant::now();
    }
}

lazy_static! {
    static ref TUI_CONSOLE: TuiConsole = TuiConsole::new();
}

lazy_static! {
    static ref PLAIN_CONSOLE: PlainConsole = PlainConsole::new();
}

pub fn initialize(mode: &Option<Mode>) {
    if !matches!(mode, Some(Mode::Tui)) {
        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::TRACE)
                .finish(),
        )
        .unwrap();
    }
}

pub fn refresh<'a>(mode: &Option<Mode>, tethers: impl Iterator<Item = &'a Tether>) {
    match mode {
        Some(Mode::Tui) => TUI_CONSOLE.refresh(tethers),
        _ => PLAIN_CONSOLE.refresh(tethers),
    }
}
