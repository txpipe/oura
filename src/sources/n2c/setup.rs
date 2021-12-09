use std::{net::TcpStream, ops::Deref, os::unix::net::UnixStream, sync::mpsc::Sender};

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    handshake::{n2c, MAINNET_MAGIC},
    localstate::{
        queries::{QueryV10, RequestV10},
        OneShotClient,
    },
    machines::{primitives::Point, run_agent},
    multiplexer::{Channel, Multiplexer},
};

use serde_derive::Deserialize;

use crate::{
    framework::{BootstrapResult, Event, SourceConfig},
    sources::common::{AddressArg, BearerKind, MagicArg},
};

use super::observe_forever;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,
}

fn do_handshake(channel: &mut Channel, magic: u64) -> Result<(), crate::framework::Error> {
    let versions = n2c::VersionTable::only_v10(magic);
    let agent = run_agent(n2c::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2c::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version".into()),
    }
}

fn find_end_of_chain(channel: &mut Channel) -> Result<Point, crate::framework::Error> {
    let agent = OneShotClient::<QueryV10>::initial(None, RequestV10::GetChainPoint);
    let agent = run_agent(agent, channel)?;
    info!("chain point query output: {:?}", agent.output);

    match agent.output {
        Some(Ok(data)) => Ok(data.try_into()?),
        Some(Err(_)) => Err("failure acquiring end of chain".into()),
        None => todo!(),
    }
}

fn setup_unix_multiplexer(path: &str) -> Result<Multiplexer, crate::framework::Error> {
    let unix = UnixStream::connect(path)?;

    Multiplexer::setup(unix, &[0, 5, 7])
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, crate::framework::Error> {
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_nodelay(true).unwrap();
    tcp.set_keepalive_ms(Some(30_000u32)).unwrap();

    Multiplexer::setup(tcp, &[0, 5, 7])
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

        let mut ls_channel = muxer.use_channel(7);
        let point = find_end_of_chain(&mut ls_channel)?;

        let cs_channel = muxer.use_channel(5);
        let handle = std::thread::spawn(move || {
            observe_forever(cs_channel, point, output).unwrap();
        });

        Ok(handle)
    }
}
