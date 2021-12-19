use std::{net::TcpStream, ops::Deref, os::unix::net::UnixStream, sync::mpsc::Sender};

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    handshake::{n2n, MAINNET_MAGIC},
    machines::run_agent,
    multiplexer::{Channel, Multiplexer},
};

use serde_derive::Deserialize;

use crate::{
    framework::{BootstrapResult, ChainWellKnownInfo, Error, Event, SourceConfig},
    sources::{
        common::{find_end_of_chain, AddressArg, BearerKind, MagicArg},
        n2n::{fetch_blocks_forever, observe_headers_forever},
    },
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,

    pub well_known: Option<ChainWellKnownInfo>,
}

fn do_handshake(channel: &mut Channel, magic: u64) -> Result<(), Error> {
    let versions = n2n::VersionTable::v6_and_above(magic);
    let agent = run_agent(n2n::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2n::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version".into()),
    }
}

fn setup_unix_multiplexer(path: &str) -> Result<Multiplexer, Error> {
    let unix = UnixStream::connect(path)?;

    Multiplexer::setup(unix, &[0, 2, 3])
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, Error> {
    let tcp = TcpStream::connect(address)?;
    tcp.set_nodelay(true)?;
    tcp.set_keepalive_ms(Some(30_000u32))?;

    Multiplexer::setup(tcp, &[0, 2, 3])
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

        let mut cs_channel = muxer.use_channel(2);

        let node_tip = find_end_of_chain(&mut cs_channel, &well_known)?;

        info!("node tip: {:?}", &node_tip);

        let (headers_tx, headers_rx) = std::sync::mpsc::channel();

        let cs_events = output.clone();
        let cs_chain_info = well_known.clone();
        let cs_handle = std::thread::spawn(move || {
            observe_headers_forever(cs_channel, cs_chain_info, node_tip, cs_events, headers_tx)
                .expect("chainsync loop failed");
        });

        let bf_channel = muxer.use_channel(3);
        let bf_chain_info = well_known.clone();
        let _bf_handle = std::thread::spawn(move || {
            fetch_blocks_forever(bf_channel, bf_chain_info, headers_rx, output)
                .expect("blockfetch loop failed");
        });

        Ok(cs_handle)
    }
}
