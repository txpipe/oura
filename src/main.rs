use clap::{value_t, App};
use log::{debug, error, info};
use net2::TcpStreamExt;
use pallas::{
    ledger::alonzo::Block,
    ouroboros::network::{
        chainsync::{BlockBody, ClientConsumer, Storage},
        handshake::n2c,
        localstate::{
            queries::{QueryV10, RequestV10},
            OneShotClient,
        },
        machines::{primitives::Point, run_agent},
        multiplexer::{Channel, Multiplexer},
    },
};
use std::{error::Error, net::TcpStream, os::unix::net::UnixStream};

#[derive(Debug)]
pub struct AlonzoLogger {}

impl Storage<BlockBody> for AlonzoLogger {
    fn save_block(&self, content: &BlockBody) -> Result<(), Box<dyn std::error::Error>> {
        let BlockBody(bytes) = content;

        let block = Block::try_from(&bytes[..]);

        match block {
            Ok(block) => {
                println!(
                    "[BLOCK] number: {}, slot: {}",
                    block.header.header_body.block_number, block.header.header_body.slot
                );
                for (idx, tx) in block.transaction_bodies.iter().enumerate() {
                    let aux = block.auxiliary_data_set.get(&(idx as u32));
                    let certs = tx
                        .certificates
                        .as_deref()
                        .and_then(|cs| Some(cs.len()))
                        .unwrap_or(0);

                    let witness = &block.transaction_witness_sets[idx];
                    let native = witness
                        .native_script
                        .as_deref()
                        .and_then(|s| Some(s.len()))
                        .unwrap_or(0);
                    let plutus = witness
                        .plutus_script
                        .as_deref()
                        .and_then(|s| Some(s.len()))
                        .unwrap_or(0);

                    println!(
                        "[TX] inputs: {}, tx outputs: {}, tx fee: {}, aux data: {}, native scripts: {}, plutus scripts: {}",
                        tx.inputs.len(), tx.outputs.len(), tx.fee, aux.is_some(), native, plutus);

                    // println!(
                    //     "[TX] tx inputs: {}, tx outputs: {}, tx fee: {}, aux: {}, certs: {}, native: {}, plutus: {}",
                    //     block.header.header_body.slot, block.header.header_body.block_number, tx.inputs.len(), tx.outputs.len(), tx.fee, aux.is_some(), certs, native, plutus);
                }
            }
            Err(err) => {
                log::error!("{:?}", err);
                log::info!("{}", hex::encode(bytes));
                panic!();
            }
        }

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

fn follow_tail(channel: Channel, from: Point) -> Result<(), Box<dyn Error>> {
    let agent = ClientConsumer::initial(vec![from], AlonzoLogger {});
    let agent = run_agent(agent, channel)?;
    error!("chainsync agent final state: {:?}", agent.state);

    Ok(())
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

fn bootstrap(socket: &str, magic: u64) -> Result<(), Box<dyn Error>> {
    //let mut muxer = setup_unix_multiplexer(socket)?;
    let mut muxer = setup_tcp_multiplexer(socket)?;

    let hs_channel = muxer.use_channel(0);
    do_handshake(hs_channel, magic)?;

    let ls_channel = muxer.use_channel(7);
    let point = find_end_of_chain(ls_channel)?;

    let cs_channel = muxer.use_channel(5);
    follow_tail(cs_channel, point)?;

    Ok(())
}

fn main() {
    env_logger::init();

    let app = App::new("app")
        .arg_from_usage("--socket=[socket] 'path of the socket'")
        .arg_from_usage("--magic=[magic] 'network magic'")
        //.arg_from_usage("--slot=[slot] 'start point slot'")
        //.arg_from_usage("--hash=[hash] 'start point hash'")
        .get_matches();

    let socket = value_t!(app, "socket", String).unwrap_or_else(|e| e.exit());
    let magic = value_t!(app, "magic", u64).unwrap_or_else(|e| e.exit());
    //let slot = value_t!(app, "slot", u64).unwrap_or_else(|e| e.exit());
    //let hash = value_t!(app, "hash", String).unwrap_or_else(|e| e.exit());
    //let point = Point(slot, hex::decode(&hash).unwrap());

    bootstrap(&socket, magic).unwrap();
}
