mod connection;
mod crazyradio_server;
mod error;
mod jsonrpc_types;
mod radio_thread;

use crate::crazyradio_server::CrazyradioServer;
use crate::error::Result;
use crazyradio::Crazyradio;

fn main() -> Result<()> {
    println!("Openning Crazyradio ...");
    let cr: Crazyradio = Crazyradio::open_first()?;

    println!("Opened Crazyradio with serial number {}", cr.serial()?);

    println!("Serving a ØMQ REP socker on port 7777...");
    let context = zmq::Context::new();
    let mut server = CrazyradioServer::new(cr, context, 7777);
    server.run();

    unreachable!();
}
