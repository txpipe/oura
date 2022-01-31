#[cfg(target_family = "unix")]
use std::os::unix::net::UnixStream;
use std::{net::TcpStream, ops::Deref};

use net2::TcpStreamExt;

use log::info;

use pallas::ouroboros::network::{
    handshake::{n2c, MAINNET_MAGIC},
    machines::{primitives::Point, run_agent},
    multiplexer::{Channel, Multiplexer},
};

use serde::Deserialize;

use crate::{
    mapper::{Config as MapperConfig, EventWriter},
    pipelining::{new_inter_stage_channel, PartialBootstrapResult, SourceProvider},
    sources::{
        common::{AddressArg, BearerKind, MagicArg, PointArg},
        define_start_point,
    },
    utils::{ChainWellKnownInfo, WithUtils},
    Error,
};

use super::run::observe_forever;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: AddressArg,

    #[serde(deserialize_with = "crate::sources::common::deserialize_magic_arg")]
    pub magic: Option<MagicArg>,

    pub since: Option<PointArg>,

    #[deprecated(note = "chain info is now pipeline-wide, use utils")]
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

impl SourceProvider for WithUtils<Config> {
    fn bootstrap(&self) -> PartialBootstrapResult {
        let (output_tx, output_rx) = new_inter_stage_channel(None);

        let mut muxer = match self.inner.address.0 {
            BearerKind::Tcp => setup_tcp_multiplexer(&self.inner.address.1)?,
            #[cfg(target_family = "unix")]
            BearerKind::Unix => setup_unix_multiplexer(&self.inner.address.1)?,
        };

        let magic = match &self.inner.magic {
            Some(m) => *m.deref(),
            None => MAINNET_MAGIC,
        };

        let writer = EventWriter::new(output_tx, self.utils.clone(), self.inner.mapper.clone());

        let mut hs_channel = muxer.use_channel(0);
        do_handshake(&mut hs_channel, magic)?;

        let mut cs_channel = muxer.use_channel(5);

        let since: Point = define_start_point(&self.inner.since, &self.utils, &mut cs_channel)?;

        info!("starting from chain point: {:?}", &since);

        let handle = std::thread::spawn(move || {
            observe_forever(cs_channel, writer, since).expect("chainsync loop failed");
        });

        Ok((handle, output_rx))
    }
}
