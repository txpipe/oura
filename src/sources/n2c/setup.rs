use std::{net::TcpStream, ops::Deref, sync::mpsc};
#[cfg(target_family = "unix")]
use std::os::unix::net::UnixStream;

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    handshake::{n2c, MAINNET_MAGIC},
    machines::{primitives::Point, run_agent},
    multiplexer::{Channel, Multiplexer},
};

use serde_derive::Deserialize;

use crate::{
    framework::{ChainWellKnownInfo, Error, EventWriter, PartialBootstrapResult, SourceConfig},
    mapping::MapperConfig,
    sources::common::{find_end_of_chain, AddressArg, BearerKind, MagicArg, PointArg},
};

use super::observe_forever;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,

    pub since: Option<PointArg>,

    pub well_known: Option<ChainWellKnownInfo>,

    #[serde(default)]
    pub mapper: MapperConfig,
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

#[cfg(target_family = "unix")]
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
    fn bootstrap(&self) -> PartialBootstrapResult {
        let (output_tx, output_rx) = mpsc::channel();

        let mut muxer = match self.address.0 {
            BearerKind::Tcp => setup_tcp_multiplexer(&self.address.1)?,
            #[cfg(target_family = "unix")]
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

        let writer = EventWriter::new(output_tx, well_known.clone().into(), self.mapper.clone());

        let mut hs_channel = muxer.use_channel(0);
        do_handshake(&mut hs_channel, magic)?;

        let mut cs_channel = muxer.use_channel(5);

        let since: Point = match &self.since {
            Some(arg) => arg.try_into()?,
            None => find_end_of_chain(&mut cs_channel, &well_known)?,
        };

        info!("starting from chain point: {:?}", &since);

        let handle = std::thread::spawn(move || {
            observe_forever(cs_channel, writer, since).expect("chainsync loop failed");
        });

        Ok((handle, output_rx))
    }
}
