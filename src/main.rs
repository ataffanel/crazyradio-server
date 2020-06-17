mod crazyradio_server;
mod jsonrpc_types;
mod error;

use crate::crazyradio_server::CrazyradioServer;
use crazyradio::Crazyradio;
use crate::error::Error;

fn main() -> Result<(), Error> {
    println!("Openning Crazyradio ...");
    let cr: Crazyradio = Crazyradio::open_first()?;

    println!("Opened Crazyradio with serial number {}", cr.serial()?);

    println!("Serving a ØMQ REP socker on port 7777...");
    let context = zmq::Context::new();
    let mut server = CrazyradioServer::new(cr, context, 7777);
    server.run();

    unreachable!();
}
