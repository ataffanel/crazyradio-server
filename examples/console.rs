// Scann for Crazyflies, connect to the first one found print the console
// This is the equivalent to the crazyradio's console example
use zmq;
use serde::{Deserialize, Serialize};
use std::str;

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
}

fn scan_channels(socket: &zmq::Socket, start: u8, stop: u8, message: &[u8]) -> Vec<u8> {
    let request = Request {
        version: "1".to_string(),
        command: Command::Scan { start, stop, message: message.to_vec() },
    };
    let request = serde_json::to_string(&request).unwrap();
    socket.send(&request, 0).unwrap();
    
    let response = socket.recv_string(0).unwrap().unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();

    match response.ret {
        Ret::Scan{found} => found,
        _ => vec![],
    }
}

fn send_packet(socket: &zmq::Socket, channel: u8, data: &[u8]) -> (bool, Vec<u8>) {
    let request = Request {
        version: "1".to_string(),
        command: Command::SendPacket { channel, data: data.to_vec() },
    };
    let request = serde_json::to_string(&request).unwrap();
    socket.send(&request, 0).unwrap();
    
    let response = socket.recv_string(0).unwrap().unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();

    match response.ret {
        Ret::SendPacket{ack_received, ack_data} => (ack_received, ack_data),
        _ => (false, vec![]),
    }
}

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    println!("Connecting to server ...");
    socket
        .connect("tcp://localhost:7777")
        .expect("failed listenning on tcp://*:7777");
    
    println!("Scanning for Crazflies ...");
    let found = scan_channels(&socket, 0, 125, &[0xff]);
    println!("Found {} crazyflies!", found.len());

    println!("Sending Packet");
    let (ack, data) = send_packet(&socket, found[0], &[0xff]);
    println!("acked: {}, data: {:?}", ack, data);

    if found.len() > 0 {
        println!("{} Crazyflies found, connecting {:?}.", found.len(), found[0]);
    
        println!("Fetching and displaying up to 100 console packets:");
        println!("==================================================");
        for _i in 1..1000 {
            let (ack, data) = send_packet(&socket, found[0], &[0xff]);
            if ack && data.len() > 0 && data[0] == 0 {
                print!("{}", str::from_utf8(&data[1..]).unwrap());
            }
        }
    } else {
        println!("No Crazyflie found!");
    }
}