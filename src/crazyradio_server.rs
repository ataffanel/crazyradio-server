#![allow(bare_trait_objects)]

use crate::error::Result;
use crate::jsonrpc_types::{Methods, Request, Response, ResponseBody, Results};
use crate::radio_thread::{radio_loop, RadioCommand, ScanResult, SendPacketResult};
use crazyradio::{Channel, Crazyradio};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct CrazyradioServer {
    socket: zmq::Socket,
    radio_cmd: Sender<RadioCommand>,
    scan_res_sender: Sender<Result<ScanResult>>,
    scan_res: Receiver<Result<ScanResult>>,
    send_packet_res_sender: Sender<Result<SendPacketResult>>,
    send_packet_res: Receiver<Result<SendPacketResult>>,
}

impl CrazyradioServer {
    pub fn new(crazyradio: Crazyradio, context: zmq::Context, port: u32) -> Self {
        let socket = context.socket(zmq::REP).unwrap();
        let listenning_uri = format!("tcp://*:{}", port);
        socket
            .bind(&listenning_uri)
            .expect(&format!("failed listenning on {}", listenning_uri));

        // Launch radio thread
        let (radio_cmd, radio_cmd_receiver) = unbounded();
        std::thread::spawn(move || {
            radio_loop(crazyradio, radio_cmd_receiver);
        });

        let (scan_res_sender, scan_res) = unbounded();
        let (send_packet_res_sender, send_packet_res) = unbounded();

        CrazyradioServer {
            socket,
            radio_cmd,
            scan_res_sender,
            scan_res,
            send_packet_res_sender,
            send_packet_res,
        }
    }

    pub fn run(&mut self) {
        loop {
            let request = self.socket.recv_string(0).unwrap().unwrap();

            let response = self.handle_request(&request);

            self.socket.send(&response, 0).unwrap();
        }
    }

    fn run_method(&mut self, method: Methods) -> Result<Results> {
        let result = match method {
            Methods::GetVersion => {
                let version = env!("CARGO_PKG_VERSION").to_string();
                Results::GetVersion(version)
            }
            Methods::Scan {
                start,
                stop,
                payload,
            } => {
                self.radio_cmd
                    .send(RadioCommand::Scan {
                        client: self.scan_res_sender.clone(),
                        start: Channel::from_number(start).unwrap(),
                        stop: Channel::from_number(stop).unwrap(),
                        payload: payload,
                    })
                    .unwrap();

                let result = self.scan_res.recv().unwrap()?;

                Results::Scan {
                    found: result.found,
                }
            }
            Methods::SendPacket { channel, payload } => {
                self.radio_cmd.send(RadioCommand::SendPacket {
                    client: self.send_packet_res_sender.clone(),
                    channel: Channel::from_number(channel).unwrap(),
                    payload: payload,
                }).unwrap();

                let result = self.send_packet_res.recv().unwrap()?;

                Results::SendPacket {
                    acked: result.acked,
                    payload: result.payload,
                }
            }
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
            |error| ResponseBody::Error {
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
