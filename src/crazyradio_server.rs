#![allow(bare_trait_objects)]

use crate::jsonrpc_types::{Methods, Request, Response, ResponseBody, Results};
use crazyradio::{Crazyradio, Channel};

pub struct CrazyradioServer {
    crazyradio: Crazyradio,
    socket: zmq::Socket,
}

impl CrazyradioServer {
    pub fn new(crazyradio: Crazyradio, context: zmq::Context, port: u32) -> Self {
        let socket = context.socket(zmq::REP).unwrap();
        let listenning_uri = format!("tcp://*:{}", port);
        socket
            .bind(&listenning_uri)
            .expect(&format!("failed listenning on {}", listenning_uri));

        CrazyradioServer { crazyradio, socket }
    }

    pub fn run(&mut self) {
        loop {
            let request = self.socket.recv_string(0).unwrap().unwrap();

            let response = self.handle_request(&request);

            self.socket.send(&response, 0).unwrap();
        }
    }

    fn run_method(&mut self, method: Methods) -> Result<Results, crate::Error> {
        let result = match method {
            Methods::GetVersion => {
                let version = env!("CARGO_PKG_VERSION").to_string();
                Results::GetVersion(version)
            },
            Methods::Scan { start, stop, payload } => {
                let found = self.crazyradio.scan_channels(
                    Channel::from_number(start).unwrap(),
                    Channel::from_number(stop).unwrap(),
                    &payload,
                )?;

                Results::Scan{
                    found: found.into_iter().map(|ch| ch.into()).collect(),
                }
            },
            Methods::SendPacket { channel, payload } => {
                let mut ack_data = Vec::new();
                ack_data.resize(32, 0);
                self.crazyradio.set_channel(Channel::from_number(channel)?)?;
    
                let ack = self.crazyradio.send_packet(&payload, &mut ack_data)?;
                ack_data.resize(ack.length, 0);
    
                Results::SendPacket {
                    acked: ack.received,
                    payload: ack_data
                }
            },
        };

        Ok(result)
    }

    /// Handle a json request and returns a json answer
    /// This function is designed to handle all error case and so will always
    /// return a valid json-formated jsonrpc2 response
    pub fn handle_request(&mut self, request: &str) -> String {
        // Deserialize request
        let request: Request = match serde_json::from_str(request) {
            Ok(r) => r,
            Err(e) => {
                return serde_json::to_string(&Response {
                    jsonrpc: "2.0".to_string(),
                    body: ResponseBody::Error {
                        code: -32700,
                        message: e.to_string(),
                    },
                    id: None,
                })
                .unwrap();
            }
        };

        // Execute request, generate a response_body
        let body = self.run_method(request.method).map_or_else(
            |error| ResponseBody::Error{
                code: 1,
                message: error.to_string(),
            },
            |result| ResponseBody::Result(result),
        );

        let response = Response {
            jsonrpc: "2.0".to_string(),
            body,
            id: request.id,
        };
        serde_json::to_string(&response).unwrap()
    }
}
