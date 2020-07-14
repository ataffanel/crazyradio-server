mod connection;
mod crazyradio_server;
mod error;
mod jsonrpc_types;
mod radio_thread;

use crate::crazyradio_server::CrazyradioServer;
use crate::error::Result;
use clap::{crate_authors, crate_version, Clap};
use crazyradio::Crazyradio;

use log::info;

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
    /// Print info logs. Use twice to log debug messages.
    #[clap(short, long, parse(from_occurrences))]
    verbose: u32,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    // Set log level from command line option unless RUST_LOG is already set
    if std::env::var_os("RUST_LOG").is_none() {
        let log_level = match opts.verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        };
        std::env::set_var("RUST_LOG", log_level);
    }

    pretty_env_logger::init();

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

    info!("Opened Crazyradio with serial number {}", cr.serial()?);

    info!("Serving a ØMQ REP socker on port {}...", opts.port);
    let context = zmq::Context::new();
    let mut server = CrazyradioServer::new(cr, context, opts.port);
    server.run();

    unreachable!();
}
