use std::{net::TcpStream, ops::Deref, os::unix::net::UnixStream, sync::mpsc::Sender};

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    chainsync::TipFinder,
    handshake::{n2n, MAINNET_MAGIC, TESTNET_MAGIC},
    machines::{primitives::Point, run_agent},
    multiplexer::{Channel, Multiplexer},
};

use serde_derive::Deserialize;

use crate::{
    framework::{BootstrapResult, Error, Event, SourceConfig},
    sources::{
        common::{AddressArg, BearerKind, MagicArg},
        n2n::{fetch_blocks_forever, observe_headers_forever},
    },
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,
}

fn do_handshake(channel: &mut Channel, magic: u64) -> Result<(), crate::framework::Error> {
    let versions = n2n::VersionTable::v6_and_above(magic);
    let agent = run_agent(n2n::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2n::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version".into()),
    }
}

fn setup_unix_multiplexer(path: &str) -> Result<Multiplexer, crate::framework::Error> {
    let unix = UnixStream::connect(path)?;

    Multiplexer::setup(unix, &[0, 2, 3])
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, crate::framework::Error> {
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_nodelay(true).unwrap();
    tcp.set_keepalive_ms(Some(30_000u32)).unwrap();

    Multiplexer::setup(tcp, &[0, 2, 3])
}

fn get_wellknonwn_chain_point(magic: u64) -> Result<Point, Error> {
    match magic {
        MAINNET_MAGIC => Ok(Point(
            4492799,
            hex::decode("f8084c61b6a238acec985b59310b6ecec49c0ab8352249afd7268da5cff2a457")?,
        )),
        TESTNET_MAGIC => Ok(Point(
            1598399,
            hex::decode("7e16781b40ebf8b6da18f7b5e8ade855d6738095ef2f1c58c77e88b6e45997a4")?,
        )),
        _ => Err("don't have a well-known chain point for the requested magic".into()),
    }
}

fn find_end_of_chain(
    channel: &mut Channel,
    wellknown_point: Point,
) -> Result<Point, crate::framework::Error> {
    let agent = TipFinder::initial(wellknown_point);
    let agent = run_agent(agent, channel)?;
    info!("chain point query output: {:?}", agent.output);

    match agent.output {
        Some(tip) => Ok(tip.0),
        None => Err("failure acquiring end of chain".into()),
    }
}

impl SourceConfig for Config {
    fn bootstrap(&self, output: Sender<Event>) -> BootstrapResult {
        let mut muxer = match self.address.0 {
            BearerKind::Tcp => setup_tcp_multiplexer(&self.address.1)?,
            BearerKind::Unix => setup_unix_multiplexer(&self.address.1)?,
        };

        let magic = match &self.magic {
            Some(m) => *m.deref(),
            None => MAINNET_MAGIC,
        };

        let mut hs_channel = muxer.use_channel(0);
        do_handshake(&mut hs_channel, magic)?;

        let mut cs_channel = muxer.use_channel(2);

        let wellknown_point = get_wellknonwn_chain_point(magic)?;
        let node_tip = find_end_of_chain(&mut cs_channel, wellknown_point)?;

        info!("node tip: {:#?}", &node_tip);

        let (headers_tx, headers_rx) = std::sync::mpsc::channel();

        let cs_events = output.clone();
        let cs_handle = std::thread::spawn(move || {
            observe_headers_forever(cs_channel, node_tip, cs_events, headers_tx).unwrap();
        });

        let bf_channel = muxer.use_channel(3);

        let _bf_handle = std::thread::spawn(move || {
            fetch_blocks_forever(bf_channel, headers_rx, output).unwrap();
        });

        Ok(cs_handle)
    }
}
