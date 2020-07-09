#![allow(bare_trait_objects)]

use crate::connection::{Connection, ConnectionStatus};
use crate::error::Result;
use crate::jsonrpc_types::{Methods, Request, Response, ResponseBody, Results};
use crate::radio_thread::RadioThread;
use crazyradio::{Channel, Crazyradio};
use std::collections::HashMap;

pub struct CrazyradioServer {
    socket: zmq::Socket,
    radio: RadioThread,
    connections: HashMap<Channel, Connection>,
}

impl CrazyradioServer {
    pub fn new(crazyradio: Crazyradio, context: zmq::Context, port: u32) -> Self {
        // Create and bind ZMQ socket
        let socket = context.socket(zmq::REP).unwrap();
        let listenning_uri = format!("tcp://*:{}", port);
        socket
            .bind(&listenning_uri)
            .expect(&format!("failed listenning on {}", listenning_uri));

        // Launch radio thread
        let radio = RadioThread::new(crazyradio);

        // No connections for now
        let connections = HashMap::new();

        CrazyradioServer {
            socket,
            radio,
            connections,
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
                let result = self.radio.scan(
                    Channel::from_number(start)?,
                    Channel::from_number(stop)?,
                    payload,
                )?;

                Results::Scan {
                    found: result.into_iter().map(|ch| ch.into()).collect(),
                }
            }
            Methods::SendPacket { channel, payload } => {
                let (ack, payload) = self
                    .radio
                    .send_packet(Channel::from_number(channel)?, payload)?;

                Results::SendPacket {
                    acked: ack.received,
                    payload: payload,
                }
            }
            Methods::Connect { channel } => {
                let channel = Channel::from_number(channel)?;

                if let Some(connection) = self.connections.get(&channel) {
                    if matches!(connection.status(), ConnectionStatus::Disconnected(_)) {
                        return Err(crate::error::Error::ArgumentError(
                            format!("Connection already active!")
                        ))
                    }
                }
                self.connections.remove(&channel);

                let connection = Connection::new(self.radio.clone(), channel);

                let (connected, status) = match connection.status() {
                    ConnectionStatus::Connecting => (false, "Connecting".to_string()),
                    ConnectionStatus::Connected => (true, "Connected".to_string()),
                    ConnectionStatus::Disconnected(message) => {
                        (false, format!("Disconnected: {}", message))
                    }
                };

                self.connections.insert(channel, connection);

                Results::Connect { connected, status }
            }
            Methods::GetConnectionStatus { channel } => {
                let channel = Channel::from_number(channel)?;
                if let Some(connection) = self.connections.get(&channel) {
                    let (connected, status) = match connection.status() {
                        ConnectionStatus::Connecting => (false, "Connecting".to_string()),
                        ConnectionStatus::Connected => (true, "Connected".to_string()),
                        ConnectionStatus::Disconnected(message) => {
                            (false, format!("Disconnected: {}", message))
                        }
                    };

                    Results::GetConnectionStatus { connected, status }
                } else {
                    let channel: u8 = channel.into();
                    return Err(crate::error::Error::ArgumentError(format!(
                        "Connection does not exist for channel {}",
                        channel
                    )));
                }
            }
            Methods::Disconnect { channel } => {
                let channel = Channel::from_number(channel)?;
                if let Some(connection) = self.connections.remove(&channel) {
                    connection.disconnect();

                    Results::Disconnect
                } else {
                    let channel: u8 = channel.into();
                    return Err(crate::error::Error::ArgumentError(format!(
                        "Connection does not exist for channel {}",
                        channel
                    )));
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
