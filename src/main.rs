mod sources;
mod sinks;
mod ports;

pub mod utils;

use clap::{value_t, App};

fn main() {
    //env_logger::init();

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

    let (tx, rx) = std::sync::mpsc::channel();

    let source = sources::chain::bootstrap(&socket, magic, tx).unwrap();
    let sink = sinks::tui::bootstrap(rx).unwrap();

    //source.join().unwrap();
    sink.join().unwrap();
}
