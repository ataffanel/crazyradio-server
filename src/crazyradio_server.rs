#![allow(bare_trait_objects)]

use crate::connection::{Connection, ConnectionStatus};
use crate::error::Result;
use crate::jsonrpc_types::{Methods, Request, Response, ResponseBody, Results};
use std::collections::HashMap;
use crazyflie_link::LinkContext;
use log::debug;

pub struct CrazyradioServer {
    socket: zmq::Socket,
    link_context: LinkContext,
    connections: HashMap<String, Connection>,
}

impl CrazyradioServer {
    pub fn new(link_context: LinkContext, context: zmq::Context, port: u32) -> Self {
        // Create and bind ZMQ socket
        let socket = context.socket(zmq::REP).unwrap();
        let listenning_uri = format!("tcp://*:{}", port);
        socket
            .bind(&listenning_uri)
            .expect(&format!("failed listenning on {}", listenning_uri));

        // No connections for now
        let connections = HashMap::new();

        CrazyradioServer {
            socket,
            link_context,
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
                address,
            } => {
                let found = self.link_context.scan(address)?;

                Results::Scan { found }
            },
            Methods::ScanSelected {
                uris,
            } => {
                let uris = uris.iter().map(String::as_str).collect();
                let found = self.link_context.scan_selected(uris)?;

                Results::ScanSelected{ found }
            },
            Methods::Connect { uri } => {
                if let Some(connection) = self.connections.get(&uri) {
                    if !matches!(connection.status(), ConnectionStatus::Disconnected(_)) {
                        dbg!(connection.status());
                        return Err(crate::error::Error::ArgumentError(format!(
                            "Connection already active!"
                        )));
                    }
                }

                self.connections.remove(&uri);

                let link = self.link_context.open_link(&uri)?;
                let connection = Connection::new(link)?;

                let (connected, status) = match connection.status() {
                    ConnectionStatus::Connected => (true, "Connected".to_string()),
                    ConnectionStatus::Disconnected(message) => {
                        (false, format!("Disconnected: {}", message))
                    }
                };

                let (pull_port, push_port) = connection.get_zmq_ports();

                self.connections.insert(uri, connection);

                Results::Connect {
                    connected,
                    status,
                    push: pull_port,
                    pull: push_port,
                }
            }
            Methods::GetConnectionStatus { uri } => {
                if let Some(connection) = self.connections.get(&uri) {
                    let (connected, status) = match connection.status() {
                        ConnectionStatus::Connected => (true, "Connected".to_string()),
                        ConnectionStatus::Disconnected(message) => {
                            (false, format!("Disconnected: {}", message))
                        }
                    };

                    Results::GetConnectionStatus { connected, status }
                } else {
                    return Err(crate::error::Error::ArgumentError(format!(
                        "Connection does not exist for uri {}",
                        &uri
                    )));
                }
            }
            Methods::Disconnect { uri } => {
                if let Some(connection) = self.connections.remove(&uri) {
                    connection.disconnect();

                    Results::Disconnect
                } else {
                    return Err(crate::error::Error::ArgumentError(format!(
                        "Connection does not exist for uri {}",
                        uri
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

        debug!("Handling request: {:#?}", request);

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
