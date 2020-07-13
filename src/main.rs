mod connection;
mod crazyradio_server;
mod error;
mod jsonrpc_types;
mod radio_thread;

use crate::crazyradio_server::CrazyradioServer;
use crate::error::Result;
use crazyradio::Crazyradio;
use clap::{Clap, crate_version, crate_authors};

/// By default, opens the first Crazyradio (ie. equivalent to `--nth 0`)
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// List connected Crazyradio serial numbers
    #[clap(short, long)]
    list: bool,
    /// Open nth Crazyradio dongle
    #[clap(short, long)]
    nth: Option<usize>,
    /// Open Crazyradio by serial number
    #[clap(short, long)]
    serial: Option<String>,
    /// Listening port for the REQ ZMQ socket
    #[clap(short, long, default_value = "7777")]
    port: u32,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    if opts.list {
        let list = Crazyradio::list_serials()?;
        for serial in list {
            println!("{}", serial);
        }
        return Ok(());
    }

    let cr = if let Some(nth) = opts.nth {
        Crazyradio::open_nth(nth)?
    } else if let Some(serial) = opts.serial {
        Crazyradio::open_by_serial(&serial)?
    } else {
        Crazyradio::open_first()?
    };

    println!("Opened Crazyradio with serial number {}", cr.serial()?);

    println!("Serving a ØMQ REP socker on port {}...", opts.port);
    let context = zmq::Context::new();
    let mut server = CrazyradioServer::new(cr, context, opts.port);
    server.run();

    unreachable!();
}
