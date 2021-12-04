mod map;

use log::{debug, error, info};
use net2::TcpStreamExt;
use pallas::{
    ledger::alonzo::Block,
    ouroboros::network::{
        chainsync::{BlockBody, ClientConsumer, Observer},
        handshake::n2c,
        localstate::{
            queries::{QueryV10, RequestV10},
            OneShotClient,
        },
        machines::{primitives::Point, run_agent},
        multiplexer::{Channel, Multiplexer},
    },
};
use std::{
    error::Error, net::TcpStream, os::unix::net::UnixStream, sync::mpsc::Sender, thread::JoinHandle,
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

                match sent {
                    Err(err) => log::error!("{:?}", err),
                    Ok(_) => (),
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
    Multiplexer::setup(unix, &vec![0, 5, 7]).into()
}

fn setup_tcp_multiplexer(address: &str) -> Result<Multiplexer, Box<dyn Error>> {
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_nodelay(true).unwrap();
    tcp.set_keepalive_ms(Some(30_000u32)).unwrap();
    Multiplexer::setup(tcp, &vec![0, 5, 7]).into()
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

pub fn bootstrap(
    socket: &str,
    magic: u64,
    sender: Sender<Event>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    //let mut muxer = setup_unix_multiplexer(socket)?;
    let mut muxer = setup_tcp_multiplexer(socket)?;

    let hs_channel = muxer.use_channel(0);
    do_handshake(hs_channel, magic)?;

    let ls_channel = muxer.use_channel(7);
    let point = find_end_of_chain(ls_channel)?;

    let cs_channel = muxer.use_channel(5);
    let handle = std::thread::spawn(move || {
        observe_forever(cs_channel, point, ChainObserver(sender)).unwrap();
    });

    Ok(handle)
}
