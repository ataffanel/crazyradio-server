#![allow(bare_trait_objects)]

use crazyradio::Crazyradio;
use crate::jsonrpc_types;

pub struct CrazyradioServer {
    crazyradio: Crazyradio,
}

impl CrazyradioServer {
    pub fn new(crazyradio: Crazyradio) -> Self {
        CrazyradioServer {
            crazyradio: crazyradio,
        }
    }

    pub fn run(&mut self) {
        println!("{}",
            serde_json::to_string_pretty(&jsonrpc_types::Request {
                jsonrpc: "2.0".to_string(),
                method: jsonrpc_types::Methods::Hello,
                id: Some(jsonrpc_types::Id::String("hello".to_string())),
            }).unwrap()
        );
        println!("{}",
            serde_json::to_string_pretty(&jsonrpc_types::Response {
                jsonrpc: "2.0".to_string(),
                body: jsonrpc_types::ResponseBody::Error {
                    code: 123,
                    message: "Error!".to_string(),
                },
                id: Some(jsonrpc_types::Id::String("hello".to_string())),
            }).unwrap()
        );
        println!("{}",
            serde_json::to_string_pretty(&jsonrpc_types::Response {
                jsonrpc: "2.0".to_string(),
                body: jsonrpc_types::ResponseBody::Result(jsonrpc_types::Result::SendPacket{acked: true, payload: vec![0xff]}),
                id: Some(jsonrpc_types::Id::String("hello".to_string())),
            }).unwrap()
        );
    }
}