// Connection handling code
use crate::error::{Error, Result};
use crossbeam_utils::sync::WaitGroup;
use log::debug;
use rand;
use rand::Rng;
use zmq::Socket;
use std::sync::{Arc, Weak};
use std::sync::RwLock;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    Connected,
    Disconnected(String),
}

pub struct Connection {
    status: Arc<RwLock<ConnectionStatus>>,
    stop: Arc<RwLock<bool>>,
    _link: Weak<crazyflie_link::Connection>,
    tx_thread: std::thread::JoinHandle<()>,
    rx_thread: std::thread::JoinHandle<()>,
    push_port: u16,
    pull_port: u16,
}

fn bind_zmq_on_random_port(socket: &zmq::Socket) -> Result<u16> {
    let n_try = 10;

    for _ in 0..n_try {
        let port = rand::thread_rng().gen_range(49152..65535);

        match socket.bind(&format!("tcp://*:{}", port)) {
            Ok(_) => return Ok(port),
            _ => continue,
        }
    }

    Err(Error::ServerError(
        "Cannot bind TCP port for connection".to_string(),
    ))
}

impl Connection {
    pub fn new(link: crazyflie_link::Connection) -> Result<Connection> {
        let link = Arc::new(link);

        let status = Arc::new(RwLock::new(ConnectionStatus::Connected));
        let stop = Arc::new(RwLock::new(false));

        // Create two ZMQ? socket for sending and receiving raw CRTP packets
        let context = zmq::Context::new();

        let socket_push = context.socket(zmq::PUSH).unwrap();
        let socket_pull = context.socket(zmq::PULL).unwrap();

        let push_port = bind_zmq_on_random_port(&socket_push)?;
        let pull_port = bind_zmq_on_random_port(&socket_pull)?;

        let connection_initialized = WaitGroup::new();

        let tx_link = link.clone();
        let tx_status = status.clone();
        let tx_stop = stop.clone();
        let tx_thread = std::thread::spawn(move || match tx_loop(tx_link, socket_pull, tx_stop) {
            Ok(()) => (),
            Err(e) => {
                *tx_status.write().unwrap() = ConnectionStatus::Disconnected(format!("Connection error: {:?}", e));
            }
        });

        let rx_link = link.clone();
        let rx_status = status.clone();
        let rx_stop = stop.clone();
        let rx_thread = std::thread::spawn(move || match rx_loop(rx_link, socket_push, rx_stop) {
            Ok(()) => (),
            Err(e) => {
                *rx_status.write().unwrap() = ConnectionStatus::Disconnected(format!("Connection error: {:?}", e));
            }
        });

        // Wait for, either, the connection being established or failed initialization
        connection_initialized.wait();

        Ok(Connection {
            status,
            stop,
            _link: Arc::downgrade(&link),
            tx_thread,
            rx_thread,
            push_port,
            pull_port,
        })
    }

    pub fn status(&self) -> ConnectionStatus {
        self.status.read().unwrap().clone()
    }

    pub fn disconnect(self) {
        *self.stop.write().unwrap() = true;
        debug!("Closing the connection!");
        self.tx_thread.join().unwrap();
        self.rx_thread.join().unwrap();
    }

    pub fn get_zmq_ports(&self) -> (u16, u16) {
        (self.pull_port, self.push_port)
    }
}

fn rx_loop(rx_link: Arc<crazyflie_link::Connection>, push_socket: Socket, stop: Arc<RwLock<bool>>) -> Result<()> {
    // Setup a quite long timeout on the push socket. If this timeout is reached, the connection is dropped.
    // This leaves a confortable time to the client to starts the socket, while making sure we will not
    // have a connection hanging if a client crash or misbehave.
    // A keepalive empty packet is sent every seconds in case there is no trafic from the Crazyflie
    push_socket.set_sndtimeo(1000)?;
    
    while *stop.read().unwrap() == false {
        let packet: Vec<u8> = match rx_link.recv_packet_timeout(Duration::from_millis(1000))? {
            Some(packet) => packet.into(),
            None => Vec::new(),
        };

        match push_socket.send(packet, 0) {
            Ok(()) => (),
            Err(zmq::Error::EAGAIN) => {
                *stop.write().unwrap() = true;
                return Err(Error::ServerError("ZMQ socket timeout".to_string()));
            },
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

fn tx_loop(tx_link: Arc<crazyflie_link::Connection>, pull_socket: Socket, stop: Arc<RwLock<bool>>) -> Result<()> {
    // Setup a timeout on zmq pull to make sure the loop checks 'stop' at regular interval
    pull_socket.set_rcvtimeo(100)?;

    while *stop.read().unwrap() == false {
        let packet = match pull_socket.recv_bytes(0) {
            Ok(packet) => packet,
            Err(zmq::Error::EAGAIN) => continue,
            Err(e) => return Err(e.into()),
        };

        tx_link.send_packet(packet.into())?;
    }

    Ok(())
}
