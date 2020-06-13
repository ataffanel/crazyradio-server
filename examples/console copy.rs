// Scann for Crazyflies, connect to the first one found print the console
// This is the equivalent to the crazyradio's console example
use zmq;

fn scan_channels(start: u8, stop: u8, message: &[u8]) -> Vec<u8> {
    todo!();
}

fn send_packet(channel: u8, data: &[u8], ack_data: &mut [u8]) -> Option<bool> {
    todo!();
}

fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    println!("Connecting to server ...");
    socket
        .connect("tcp://localhost:7777")
        .expect("failed listenning on tcp://*:7777");
    println!("Sending Scan");
    socket.send("{\"version\": \"1\", \"command\": {\"type\":\"Scan\", \"start\":0, \"stop\":125, \"message\": [255]}}", 0).unwrap();
    println!("Waiting for answer");
    let answer = socket.recv_string(0).unwrap().unwrap();
    println!("Received: {}", answer);

    println!("Sending Packet");
    socket.send("{\"version\": \"1\", \"command\": {\"type\":\"SendPacket\", \"channel\":47, \"data\": [255]}}", 0).unwrap();
    println!("Waiting for answer");
    let answer = socket.recv_string(0).unwrap().unwrap();
    println!("Received: {}", answer);
}
