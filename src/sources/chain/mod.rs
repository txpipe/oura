mod map;

use log::{error, info};
use net2::TcpStreamExt;
use pallas::{
    ledger::alonzo::Block,
    ouroboros::network::{
        chainsync::{BlockBody, ClientConsumer, Observer},
        handshake::{n2c, MAINNET_MAGIC, TESTNET_MAGIC},
        localstate::{
            queries::{QueryV10, RequestV10},
            OneShotClient,
        },
        machines::{primitives::Point, run_agent},
        multiplexer::{Channel, Multiplexer},
    },
};
use std::{
    error::Error, net::TcpStream, ops::Deref, os::unix::net::UnixStream, str::FromStr,
    sync::mpsc::Sender, thread::JoinHandle,
};

use crate::ports::Event;

use self::map::{EventSource, EventWriter};

#[derive(Debug)]
pub struct ChainObserver(pub Sender<Event>);

impl Observer<BlockBody> for ChainObserver {
    fn on_block(&self, content: &BlockBody) -> Result<(), Box<dyn std::error::Error>> {
        let BlockBody(bytes) = content;
        let block = Block::try_from(&bytes[..]);

        match block {
            Ok(block) => {
                let mut storage = Vec::with_capacity(5 + (block.transaction_bodies.len() * 2));
                let mut writer = EventWriter::new(&mut storage);
                block.write_events(&mut writer);
                let sent = storage
                    .into_iter()
                    .map(|e| self.0.send(e))
                    .collect::<Result<Vec<_>, _>>();

                if let Err(err) = sent {
                    log::error!("{:?}", err)
                }
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(bytes));
            }
        };

        Ok(())
    }

    fn on_rollback(&self, point: &Point) -> Result<(), Box<dyn std::error::Error>> {
        println!("rollback to {:#?}", point);
        Ok(())
    }
}

fn do_handshake(channel: Channel, magic: u64) -> Result<(), Box<dyn Error>> {
    let versions = n2c::VersionTable::only_v10(magic);
    let agent = run_agent(n2c::Client::initial(versions), channel)?;
    info!("handshake output: {:?}", agent.output);

    match agent.output {
        n2c::Output::Accepted(_, _) => Ok(()),
        _ => Err("couldn't agree on handshake version".into()),
    }
}

fn find_end_of_chain(channel: Channel) -> Result<Point, Box<dyn Error>> {
    let agent = OneShotClient::<QueryV10>::initial(None, RequestV10::GetChainPoint);
    let agent = run_agent(agent, channel)?;
    info!("chain point query output: {:?}", agent.output);

    match agent.output {
        Some(Ok(data)) => Ok(data.try_into()?),
        Some(Err(_)) => Err("failure acquiring end of chain".into()),
        None => todo!(),
    }
}

fn setup_unix_multiplexer(path: &str) -> Result<Multiplexer, Box<dyn Error>> {
    let unix = UnixStream::connect(path)?;

    Multiplexer::setup(unix, &[0, 5, 7])
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, Box<dyn Error>> {
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_nodelay(true).unwrap();
    tcp.set_keepalive_ms(Some(30_000u32)).unwrap();

    Multiplexer::setup(tcp, &[0, 5, 7])
}

fn observe_forever(
    channel: Channel,
    from: Point,
    observer: ChainObserver,
) -> Result<(), Box<dyn Error>> {
    let agent = ClientConsumer::initial(vec![from], observer);
    let agent = run_agent(agent, channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
}

pub enum BearerKind {
    Tcp,
    Unix,
}

pub struct AddressArg(pub BearerKind, pub String);

impl FromStr for BearerKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unix" => Ok(BearerKind::Unix),
            "tcp" => Ok(BearerKind::Tcp),
            _ => Err("can't parse bearer type value"),
        }
    }
}

pub struct MagicArg(u64);

impl Deref for MagicArg {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MagicArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "testnet" => MagicArg(TESTNET_MAGIC),
            "mainnet" => MagicArg(MAINNET_MAGIC),
            _ => MagicArg(u64::from_str(s).map_err(|_| "can't parse magic value")?),
        };

        Ok(m)
    }
}

pub enum PeerMode {
    AsNode,
    AsClient,
}

impl FromStr for PeerMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "node" => Ok(PeerMode::AsNode),
            "client" => Ok(PeerMode::AsClient),
            _ => Err("can't parse peer mode (valid values: client|node)"),
        }
    }
}

pub fn bootstrap(
    address: AddressArg,
    magic: Option<MagicArg>,
    mode: Option<PeerMode>,
    sender: Sender<Event>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let mut muxer = match address.0 {
        BearerKind::Tcp => setup_tcp_multiplexer(&address.1)?,
        BearerKind::Unix => setup_unix_multiplexer(&address.1)?,
    };

    // TODO: placeholder for when we implement chainsync + blockfetch
    let _mode = match (mode, address.0) {
        (Some(mode), _) => mode,
        (None, BearerKind::Tcp) => PeerMode::AsClient,
        (None, BearerKind::Unix) => PeerMode::AsNode,
    };

    let magic = magic.unwrap_or(MagicArg(MAINNET_MAGIC));
    let hs_channel = muxer.use_channel(0);
    do_handshake(hs_channel, *magic)?;

    let ls_channel = muxer.use_channel(7);
    let point = find_end_of_chain(ls_channel)?;

    let cs_channel = muxer.use_channel(5);
    let handle = std::thread::spawn(move || {
        observe_forever(cs_channel, point, ChainObserver(sender)).unwrap();
    });

    Ok(handle)
}
