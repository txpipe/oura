use clap::{Parser, ValueEnum};
use oura::{
    daemon::{run_daemon, ConfigRoot},
    filters,
    framework::{ChainConfig, Error, IntersectConfig},
    sinks, sources,
};
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn run(args: &Args) -> Result<(), Error> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let chain = args.magic.clone().unwrap_or_default().into();
    let intersect = parse_since(args.since.clone())?;
    let bearer = args.bearer.clone().unwrap_or_default();
    let source = match bearer {
        Bearer::Unix => sources::Config::N2C(sources::n2c::Config {
            socket_path: args.socket.clone().into(),
        }),
        Bearer::Tcp => sources::Config::N2N(sources::n2n::Config {
            peers: vec![args.socket.clone()],
        }),
    };
    let filter = filters::Config::LegacyV1(filters::legacy_v1::Config {
        include_block_end_events: true,
        ..Default::default()
    });

    let sink = match args.output.clone() {
        Some(output) => sinks::Config::FileRotate(sinks::file_rotate::Config {
            output_path: Some(output),
            ..Default::default()
        }),
        None => sinks::Config::Stdout(sinks::stdout::Config),
    };

    let config = ConfigRoot {
        source,
        filters: Some(vec![filter]),
        sink,
        intersect,
        finalize: None,
        chain: Some(chain),
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

fn parse_since(since: Option<String>) -> Result<IntersectConfig, Error> {
    match since {
        Some(since) => {
            let parts: Vec<&str> = since.split(",").collect();
            if parts.len() != 2 {
                return Err(Error::Parse("invalid since format".into()));
            }

            let slot: u64 = parts[0]
                .parse()
                .map_err(|_| Error::Parse(format!("since slot {} must be u64", parts[0])))?;
            let hash = parts[1].to_string();

            Ok(IntersectConfig::Point(slot, hash))
        }
        None => Ok(IntersectConfig::Tip),
    }
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    socket: String,

    #[arg(long)]
    bearer: Option<Bearer>,

    #[arg(long)]
    magic: Option<Chain>,

    /// point in the chain to start reading from, expects format `slot,hex-hash`
    #[arg(long)]
    since: Option<String>,

    /// output file path
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(ValueEnum, Clone, Default)]
enum Bearer {
    Unix,
    #[default]
    Tcp,
}

#[derive(ValueEnum, Clone, Default)]
enum Chain {
    #[default]
    Mainnet,
    Testnet,
    PreProd,
    Preview,
}
impl From<Chain> for ChainConfig {
    fn from(value: Chain) -> Self {
        match value {
            Chain::Mainnet => ChainConfig::Mainnet,
            Chain::Testnet => ChainConfig::Testnet,
            Chain::PreProd => ChainConfig::PreProd,
            Chain::Preview => ChainConfig::Preview,
        }
    }
}
