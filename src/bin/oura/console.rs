use gasket::{daemon::Daemon, metrics::Reading, runtime::Tether};
use std::{sync::Arc, time::Duration};

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

pub async fn render(daemon: Arc<Daemon>, tui_enabled: bool) {
    if !tui_enabled {
        return;
    }

    let tui = TuiConsole::new();

    loop {
        tui.refresh(daemon.tethers());
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
