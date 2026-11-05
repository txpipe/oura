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

    // detached: dropping a tokio `JoinHandle` doesn't abort the task; both keep
    // running until the runtime is dropped below.
    tokio_rt.spawn(serve_prometheus(daemon.clone(), metrics));
    tokio_rt.spawn(console::render(daemon.clone(), args.tui));

    // The daemon is shared via `Arc` with the prometheus + tui tasks, so we poll
    // its stop condition over `&self` instead of letting `block` consume it. The
    // `StopReason` tells us why the pipeline stopped so we can pick an exit code.
    let reason = loop {
        if let Some(reason) = daemon.stop_reason() {
            break reason;
        }

        std::thread::sleep(std::time::Duration::from_millis(1500));
    };

    info!(%reason, "oura is stopping");

    // Dropping the runtime cancels the prometheus + tui tasks and waits for their
    // futures to drop, releasing their `Arc` clones so we can reclaim sole
    // ownership and tear the stages down gracefully.
    drop(tokio_rt);

    Arc::try_unwrap(daemon)
        .expect("runtime tasks released their daemon clones")
        .teardown();

    if reason.is_graceful() {
        Ok(())
    } else {
        Err(Error::custom(format!("pipeline stopped: {reason}")))
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
