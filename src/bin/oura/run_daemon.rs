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

    daemon.block();

    info!("oura is stopping");

    daemon.teardown();
    prometheus.abort();
    tui.abort();

    Ok(())
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
