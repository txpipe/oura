use gasket::daemon::Daemon;
use oura::daemon::{run_daemon, ConfigRoot, MetricsConfig};
use oura::framework::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::console;

fn setup_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();
}

async fn serve_prometheus(
    daemon: Arc<Daemon>,
    metrics: Option<MetricsConfig>,
) -> Result<(), Error> {
    if let Some(metrics) = metrics {
        info!("starting metrics exporter");
        let runtime = daemon.clone();

        let addr: SocketAddr = metrics
            .address
            .as_deref()
            .unwrap_or("0.0.0.0:9186")
            .parse()
            .map_err(Error::parse)?;

        gasket_prometheus::serve(addr, runtime).await;
    }

    Ok(())
}

pub fn run(args: &Args) -> Result<(), Error> {
    if !args.tui {
        setup_tracing();
    }

    let config = ConfigRoot::new(&args.config).map_err(Error::config)?;
    let metrics = config.metrics.clone();

    let daemon = run_daemon(config)?;

    info!("oura is running");

    let daemon = Arc::new(daemon);

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let prometheus = tokio_rt.spawn(serve_prometheus(daemon.clone(), metrics));
    let tui = tokio_rt.spawn(console::render(daemon.clone(), args.tui));

    // `block`/`teardown` consume the `Daemon`, but it's shared via `Arc` with the
    // prometheus + tui tasks, so we drive the stop loop over `&self` ourselves.
    while !daemon.should_stop() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    info!("oura is stopping");

    prometheus.abort();
    tui.abort();

    // Capture *why* the pipeline stopped before tearing down (teardown ends every
    // stage). A stage that reached `Ended` means a finalization filter completed
    // gracefully — only such filters self-end, so other stages erroring out as a
    // teardown side effect don't confuse this. Any other stop with no `Ended`
    // stage is a crashed/stalled stage, unless the user sent a termination signal.
    use gasket::runtime::{StagePhase, TetherState};
    let finalized = daemon.tethers().any(|tether| {
        matches!(
            tether.check_state(),
            TetherState::Alive(StagePhase::Ended) | TetherState::Finished(StagePhase::Ended)
        )
    });
    let terminated = daemon.is_terminated();

    // wait for the aborted tasks to drop their `Arc` clones so we can reclaim
    // sole ownership and tear the stages down gracefully.
    tokio_rt.block_on(async {
        let _ = prometheus.await;
        let _ = tui.await;
    });

    if let Ok(daemon) = Arc::try_unwrap(daemon) {
        daemon.teardown();
    }

    if finalized || terminated {
        Ok(())
    } else {
        Err(Error::custom(
            "pipeline stopped before finalizing (a stage crashed or stalled)",
        ))
    }
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// config file to load by the daemon
    #[clap(long, value_parser)]
    config: Option<std::path::PathBuf>,

    /// display the terminal UI
    #[clap(long, action)]
    tui: bool,
}
