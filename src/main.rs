mod connection;
mod crazyradio_server;
mod error;
mod jsonrpc_types;

use crate::crazyradio_server::CrazyradioServer;
use crate::error::Result;
use clap::{crate_authors, crate_version, Clap};
use crazyflie_link::LinkContext;

use log::info;

/// By default, opens the first Crazyradio (ie. equivalent to `--nth 0`)
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// Listening port for the REQ ZMQ socket
    #[clap(short, long, default_value = "7777")]
    port: u32,
    /// Print debug messages.
    #[clap(short, long, parse(from_occurrences), default_value = "0")]
    verbose: i32,
    /// Hide info messages
    #[clap(short, long, parse(from_occurrences), default_value = "0")]
    quiet: i32,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let verboseness = opts.verbose - opts.quiet;

    // Set log level from command line option unless RUST_LOG is already set
    if std::env::var_os("RUST_LOG").is_none() {
        let log_level = match verboseness {
            v if v < 0 => "warn",
            0 => "info",
            _ => "debug",
        };
        std::env::set_var("RUST_LOG", log_level);
    }

    pretty_env_logger::init();

    let link_context = LinkContext::new();

    info!("Serving a ØMQ REP socker on port {}...", opts.port);
    let context = zmq::Context::new();
    let mut server = CrazyradioServer::new(link_context, context, opts.port);
    server.run();

    unreachable!();
}
