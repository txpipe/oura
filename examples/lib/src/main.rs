use std::error::Error;

use dotenv::dotenv;
use gasket::{daemon::Daemon, messaging::tokio::connect_ports, runtime::Policy};
use oura::{
    cursor, filters,
    framework::{ChainConfig, Context, IntersectConfig},
    sources,
};
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod my_filter;
mod my_sink;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let chain = ChainConfig::Mainnet;
    //let intersect = IntersectConfig::Tip;
    let intersect = IntersectConfig::Point(
        146787124,
        "fc6fb07cebf8da13ade4d300010296294846d4d540f4e56428f6a22394f8b139".to_string(),
    );

    let current_dir = std::env::current_dir().unwrap();
    let cursor = cursor::Config::default();
    let breadcrumbs = cursor.initial_load()?;

    let ctx = Context {
        chain,
        intersect,
        finalize: None,
        current_dir,
        breadcrumbs,
    };

    // Use an existing Oura source, check the sources available in the documentation
    let source_config = sources::Config::N2N(sources::n2n::Config {
        peers: vec!["backbone.mainnet.cardanofoundation.org:3001".to_string()],
    });
    let mut source = source_config.bootstrapper(&ctx)?;

    // It's possible to use an existing Oura sink, check the sinks available in the documentation
    // let sink_config = sinks::Config::Stdout(sinks::stdout::Config);
    // let mut sink = sink_config.bootstrapper(&ctx)?;

    let filter_split_config = filters::Config::SplitBlock(filters::split_block::Config {});
    let mut filter_split = filter_split_config.bootstrapper(&ctx)?;

    let filter_parse_config = filters::Config::ParseCbor(filters::parse_cbor::Config {});
    let mut filter_parse = filter_parse_config.bootstrapper(&ctx)?;

    let mut my_filter = my_filter::Stage::default();
    let mut my_sink = my_sink::Stage::default();

    let mut cursor = cursor.bootstrapper(&ctx)?;

    connect_ports(source.borrow_output(), filter_split.borrow_input(), 100);
    connect_ports(
        filter_split.borrow_output(),
        filter_parse.borrow_input(),
        100,
    );
    connect_ports(filter_parse.borrow_output(), &mut my_filter.input, 100);
    connect_ports(&mut my_filter.output, &mut my_sink.input, 100);
    connect_ports(&mut my_sink.cursor, cursor.borrow_track(), 100);

    let policy = Policy::default();

    let daemon = Daemon(vec![
        source.spawn(policy.clone()),
        filter_parse.spawn(policy.clone()),
        filter_split.spawn(policy.clone()),
        gasket::runtime::spawn_stage(my_filter, policy.clone()),
        gasket::runtime::spawn_stage(my_sink, policy.clone()),
        //sink.spawn(policy.clone()),
    ]);

    daemon.block();

    info!("oura is stopping");

    daemon.teardown();

    Ok(())
}
