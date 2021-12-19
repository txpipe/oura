use std::{net::TcpStream, ops::Deref, os::unix::net::UnixStream, sync::mpsc::Sender};

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    handshake::{n2c, MAINNET_MAGIC},
    machines::run_agent,
    multiplexer::{Channel, Multiplexer},
};

use serde_derive::Deserialize;

use crate::{
    framework::{BootstrapResult, ChainWellKnownInfo, Error, Event, SourceConfig},
    sources::common::{find_end_of_chain, AddressArg, BearerKind, MagicArg},
};

use super::observe_forever;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,

    pub well_known: Option<ChainWellKnownInfo>,
}

fn do_handshake(channel: &mut Channel, magic: u64) -> Result<(), Error> {
    let versions = n2c::VersionTable::v1_and_above(magic);
    let agent = run_agent(n2c::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2c::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version for client connection".into()),
    }
}

fn setup_unix_multiplexer(path: &str) -> Result<Multiplexer, Error> {
    let unix = UnixStream::connect(path)?;

    Multiplexer::setup(unix, &[0, 5, 7])
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, Error> {
    let tcp = TcpStream::connect(address)?;
    tcp.set_nodelay(true)?;
    tcp.set_keepalive_ms(Some(30_000u32))?;

    Multiplexer::setup(tcp, &[0, 5])
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

        let well_known = match &self.well_known {
            Some(info) => info.clone(),
            None => ChainWellKnownInfo::try_from_magic(magic)?,
        };

        let mut hs_channel = muxer.use_channel(0);
        do_handshake(&mut hs_channel, magic)?;

        let mut cs_channel = muxer.use_channel(5);

        let node_tip = find_end_of_chain(&mut cs_channel, &well_known)?;

        info!("node tip: {:?}", &node_tip);

        let handle = std::thread::spawn(move || {
            observe_forever(cs_channel, well_known, node_tip, output).expect("chainsync loop failed");
        });

        Ok(handle)
    }
}
