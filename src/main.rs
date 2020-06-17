mod crazyradio_server;
mod jsonrpc_types;

use core::fmt::Display;
use crazyradio::{Crazyradio, Channel};
use serde::{Deserialize, Serialize};
use crate::crazyradio_server::CrazyradioServer;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    version: String,
    command: Command,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Command {
    Scan{ start: u8, stop: u8, message: Vec<u8> },
    SendPacket { channel: u8, data: Vec<u8> },
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    version: String,
    ret: Ret,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Ret {
    Scan { found: Vec<u8> },
    SendPacket { ack_received: bool, ack_data: Vec<u8> },
    Error { reason: String },
}

#[derive(Debug)]
enum Error {
    DeserializeError(serde_json::Error),
    CrazyradioError(crazyradio::Error),
}

impl Display for Error { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::DeserializeError(error)
    }
}

impl From<crazyradio::Error> for Error {
    fn from(error: crazyradio::Error) -> Self {
        Error::CrazyradioError(error)
    }
}

fn run_command(cr: &mut Crazyradio, command: Command) -> Result<Ret, Error> {
    match command {
        Command::Scan{start, stop, message} => {
            let result = cr.scan_channels(Channel::from_number(start).unwrap(), 
                                          Channel::from_number(stop).unwrap(),
                                          &message)?;
            
            Ok(Ret::Scan{
                found: result.into_iter().map(|ch| ch.into()).collect()
            })
        },
        Command::SendPacket{channel, data} => {
            let mut ack_data = Vec::new();
            ack_data.resize(32, 0);
            cr.set_channel(Channel::from_number(channel)?)?;

            let ack = cr.send_packet(&data, &mut ack_data)?;
            ack_data.resize(ack.length, 0);

           Ok(Ret::SendPacket{
                ack_received: ack.received,
                ack_data: ack_data,
            })
        },
    }
}

fn main()  -> Result<(), Error> {
    println!("Openning Crazyradio ...");
    let mut cr: Crazyradio = Crazyradio::open_first()?;

    // let cmd = Request {
    //     version: "1".to_string(),
    //     command: Command::Scan{
    //         start: 0,
    //         stop: 125,
    //         message: vec![0xff],
    //     }
    // };
    // dbg!(serde_json::to_string(&cmd).unwrap());

    println!("Opened Crazyradio with serial number {}", cr.serial()?);

    println!("Opening ZMQ REQ/REP socket on port 7777 ...");
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REP).unwrap();
    socket
        .bind("tcp://*:7777")
        .expect("failed listenning on tcp://*:7777");
    
    let mut server = CrazyradioServer::new(cr);
    server.run();

    Ok(())

    // println!("Entering main loop ...");
    // loop {
    //     let request = socket.recv_string(0).unwrap().unwrap();
    //     let request: Request = serde_json::from_str(&request).unwrap();

    //     let ret = run_command(&mut cr, request.command)
    //                   .unwrap_or_else(|e| Ret::Error{reason: e.to_string()});

    //     let response = Response {
    //         version: "1".to_string(),
    //         ret
    //     };
    //     socket.send(&serde_json::to_string(&response).unwrap(), 0).unwrap();
        
    // }
}
