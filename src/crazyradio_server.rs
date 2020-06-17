#![allow(bare_trait_objects)]

use crazyradio::Crazyradio;
use easy_jsonrpc::Handler;
use serde_json::json;
use std::cell::RefCell;



#[easy_jsonrpc::rpc]
pub trait Rpc {
    fn hello(&self) -> String;
    fn version(&self) -> String {
        format!("{}", env!("CARGO_PKG_VERSION"))
    }

    fn send_packet(&self, channel: u8, packet: Vec<u8>) ->  Option<(bool, Vec<u8>)>;
}

pub struct CrazyradioServer {
    crazyradio: RefCell<Crazyradio>,
}

impl Rpc for CrazyradioServer {
    fn hello(&self) -> String {
        let mut crazyradio = self.crazyradio.borrow_mut();
        crazyradio.send_packet(&[0xff], &mut [0; 32]).unwrap();
        format!("Hello there, I am {}!", option_env!("CARGO_PKG_VERSION").unwrap())
    }

    fn send_packet(&self, channel: u8, packet: Vec<u8>) -> Option<(bool, Vec<u8>)> {
        let mut cr = self.crazyradio.borrow_mut();

        let mut ack_data = Vec::new();
        ack_data.resize(32, 0);
        cr.set_channel(crazyradio::Channel::from_number(channel).unwrap()).unwrap();

        let ack = cr.send_packet(&packet, &mut ack_data).unwrap();
        ack_data.resize(ack.length, 0);

        Some((ack.received, ack_data))
    }
}

impl CrazyradioServer {
    pub fn new(crazyradio: Crazyradio) -> Self {
        CrazyradioServer {
            crazyradio: RefCell::new(crazyradio),
        }
    }

    pub fn run(&mut self) {
        let handler = self as &dyn Rpc;
        dbg!(
            handler.handle_request(json!({
                "jsonrpc": "2.0",
                "method": "send_packet",
                "params": [256, [255]],
                "id": 1
            }))
        );
    }
}