//! Runs real blocks at each Cardano protocol version through Oura's filter
//! stages. See `tests/protocol/README.md` for where each block comes from.

#![allow(unexpected_cfgs)]

use std::time::Duration;

use gasket::messaging::tokio::connect_ports;
use oura::filters;
use oura::framework::legacy_v1::{Era, EventData};
use oura::framework::*;
use pallas::ledger::traverse::MultiEraBlock;
use pallas::network::miniprotocols::Point;

struct Fixture {
    file: &'static str,
    hex: &'static str,
    legacy_era: Era,
}

macro_rules! fixture {
    ($file:literal, $era:expr) => {
        Fixture {
            file: $file,
            hex: include_str!(concat!("protocol/", $file)),
            legacy_era: $era,
        }
    };
}

fn released() -> Vec<(u64, Fixture)> {
    vec![
        (7, fixture!("babbage-v7.block", Era::Babbage)),
        (8, fixture!("babbage-v8.block", Era::Babbage)),
        (9, fixture!("conway-v9.block", Era::Conway)),
        (10, fixture!("conway-v10.block", Era::Conway)),
        (11, fixture!("conway-v11.block", Era::Conway)),
        (12, fixture!("conway-v12.block", Era::Conway)),
    ]
}

impl Fixture {
    fn bytes(&self) -> Vec<u8> {
        hex::decode(self.hex.trim()).unwrap()
    }
}

fn header_version(block: &MultiEraBlock) -> u64 {
    if let Some(block) = block.as_conway() {
        block.header.header_body.protocol_version.0
    } else {
        block
            .as_babbage()
            .unwrap()
            .header
            .header_body
            .protocol_version
            .0
    }
}

fn context() -> Context {
    Context {
        chain: ChainConfig::default(),
        intersect: IntersectConfig::Origin,
        current_dir: std::env::temp_dir(),
        breadcrumbs: Breadcrumbs::from_points(vec![], 1),
    }
}

fn policy() -> gasket::runtime::Policy {
    let retry = gasket::retries::Policy {
        max_retries: 0,
        backoff_unit: Duration::from_millis(1),
        backoff_factor: 1,
        max_backoff: Duration::from_millis(1),
        dismissible: false,
    };

    gasket::runtime::Policy {
        tick_timeout: None,
        bootstrap_retry: retry.clone(),
        work_retry: retry.clone(),
        teardown_retry: retry,
    }
}

/// Sends one block through a running stage and collects what comes out, up to
/// `expected` events. A stage that fails on the block produces nothing, so the
/// count falls short and the caller's assertion catches it.
async fn run_stage(config: filters::Config, block: Vec<u8>, expected: usize) -> Vec<ChainEvent> {
    let mut stage = config.bootstrapper(&context()).unwrap();

    let mut input = FilterOutputPort::default();
    let mut output = FilterInputPort::default();
    connect_ports(&mut input, stage.borrow_input(), 100);
    connect_ports(stage.borrow_output(), &mut output, 100);

    let tether = stage.spawn(policy());

    input
        .send(ChainEvent::apply(Point::Origin, Record::CborBlock(block)))
        .await
        .unwrap();

    let mut events = Vec::new();

    while events.len() < expected {
        match tokio::time::timeout(Duration::from_secs(5), output.recv()).await {
            Ok(Ok(message)) => events.push(message.payload),
            _ => break,
        }
    }

    tether.dismiss_stage().ok();

    events
}

async fn parses_into_a_block(fixture: &Fixture) {
    let bytes = fixture.bytes();
    let txs = MultiEraBlock::decode(&bytes).unwrap().txs().len();

    let events = run_stage(
        filters::Config::ParseCbor(filters::parse_cbor::Config::default()),
        bytes,
        1,
    )
    .await;

    match events.first().and_then(ChainEvent::record) {
        Some(Record::ParsedBlock(block)) => {
            assert_eq!(
                block.body.as_ref().unwrap().tx.len(),
                txs,
                "{}",
                fixture.file
            );
        }
        _ => panic!("{}: no parsed block", fixture.file),
    }
}

async fn splits_into_txs(fixture: &Fixture) {
    let bytes = fixture.bytes();
    let txs = MultiEraBlock::decode(&bytes).unwrap().txs().len();

    let events = run_stage(
        filters::Config::SplitBlock(filters::split_block::Config::default()),
        bytes,
        txs,
    )
    .await;

    assert_eq!(events.len(), txs, "{}", fixture.file);

    for event in &events {
        assert!(
            matches!(event.record(), Some(Record::CborTx(_))),
            "{}",
            fixture.file
        );
    }
}

async fn reports_its_legacy_era(fixture: &Fixture) {
    let events = run_stage(
        filters::Config::LegacyV1(filters::legacy_v1::Config::default()),
        fixture.bytes(),
        1,
    )
    .await;

    match events.first().and_then(ChainEvent::record) {
        Some(Record::OuraV1Event(event)) => match &event.data {
            EventData::Block(block) => {
                assert_eq!(block.era, fixture.legacy_era, "{}", fixture.file)
            }
            other => panic!("{}: first event was {other:?}", fixture.file),
        },
        _ => panic!("{}: no legacy event", fixture.file),
    }
}

#[test]
fn fixtures_carry_the_version_they_claim() {
    for (version, fixture) in released() {
        let bytes = fixture.bytes();
        let block = MultiEraBlock::decode(&bytes).unwrap();
        assert_eq!(header_version(&block), version, "{}", fixture.file);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn parse_cbor_handles_every_version() {
    for (_, fixture) in released() {
        parses_into_a_block(&fixture).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn split_block_handles_every_version() {
    for (_, fixture) in released() {
        splits_into_txs(&fixture).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_v1_handles_every_version() {
    for (_, fixture) in released() {
        reports_its_legacy_era(&fixture).await;
    }
}

/// The Dijkstra era needs a Pallas release that can read it. The `n+1` CI job
/// builds against the pre-release and turns this on with `--cfg pallas_next`.
#[cfg(pallas_next)]
mod dijkstra {
    use super::*;

    fn dijkstra() -> Fixture {
        fixture!("dijkstra-v12.block", Era::Unknown)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_cbor_handles_dijkstra() {
        parses_into_a_block(&dijkstra()).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn split_block_handles_dijkstra() {
        splits_into_txs(&dijkstra()).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn legacy_v1_handles_dijkstra() {
        reports_its_legacy_era(&dijkstra()).await;
    }
}
