use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    daemon::{run_daemon, ConfigRoot},
    framework::{Error, IntersectConfig},
    sinks, sources,
};

pub fn run(args: &Args) -> Result<(), Error> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let source = sources::Config::Bitcoin(sources::btc::Config {
        rpc_url: args.rpc_host.clone(),
        rpc_user: args.user.clone(),
        rpc_password: args.password.clone(),
        rpc_interval: args.interval,
    });

    let sink = sinks::Config::Terminal(sinks::terminal::Config {
        throttle_min_span_millis: args.throttle,
        wrap: Some(args.wrap),
        adahandle_policy: Default::default(),
    });

    let config = ConfigRoot {
        source,
        filters: None,
        sink,
        intersect: IntersectConfig::Tip,
        finalize: None,
        chain: None,
        retries: None,
        cursor: None,
        metrics: None,
    };

    let daemon = run_daemon(config)?;

    daemon.block();

    info!("oura is stopping");

    daemon.teardown();

    Ok(())
}

#[derive(Parser, Clone)]
pub struct Args {
    /// Bitcoin RPC server
    rpc_host: String,

    /// Polling interval in seconds
    #[arg(long)]
    interval: Option<u64>,

    /// RPC server username
    #[arg(long)]
    user: Option<String>,

    /// RPC server password
    #[arg(long)]
    password: Option<String>,

    /// milliseconds to wait between output lines (for easier reading)
    #[arg(long)]
    throttle: Option<u64>,

    /// wrap output lines
    #[arg(short, long, default_value_t = false)]
    wrap: bool,
}