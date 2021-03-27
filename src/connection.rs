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
    link: Arc<crazyflie_link::Connection>,
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

        let tx_link = Arc::downgrade(&link);
        let tx_status = status.clone();
        let tx_stop = stop.clone();
        let tx_thread = std::thread::spawn(move || match tx_loop(tx_link, socket_pull, tx_stop) {
            Ok(()) => (),
            Err(e) => {
                *tx_status.write().unwrap() = ConnectionStatus::Disconnected(format!("Connection error: {:?}", e));
            }
        });

        let rx_link = Arc::downgrade(&link);
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
            link,
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
        drop(self.link);
        debug!("Closing the connection!");
        self.tx_thread.join().unwrap();
        self.rx_thread.join().unwrap();
    }

    pub fn get_zmq_ports(&self) -> (u16, u16) {
        (self.pull_port, self.push_port)
    }
}

fn rx_loop(rx_link: Weak<crazyflie_link::Connection>, push_socket: Socket, stop: Arc<RwLock<bool>>) -> Result<()> {
    push_socket.set_sndtimeo(100)?;
    loop {
        if *stop.read().unwrap() {
            return Ok(())
        }

        if let Some(link) = rx_link.upgrade() {
            let packet: Vec<u8> = match link.recv_packet_timeout(Duration::from_millis(100))? {
                Some(packet) => packet.into(),
                None => continue,
            };

            match push_socket.send(packet, 0) {
                Ok(()) => (),
                Err(zmq::Error::EAGAIN) => continue,
                Err(e) => return Err(e.into()),
            }
        } else {
            return Ok(());
        }
    }
}

fn tx_loop(tx_link: Weak<crazyflie_link::Connection>, pull_socket: Socket, stop: Arc<RwLock<bool>>) -> Result<()> {
    pull_socket.set_rcvtimeo(100)?;
    loop {
        if *stop.read().unwrap() {
            return Ok(())
        }

        if let Some(link) = tx_link.upgrade() {
            let packet = match pull_socket.recv_bytes(0) {
                Ok(packet) => packet,
                Err(zmq::Error::EAGAIN) => continue,
                Err(e) => return Err(e.into()),
            };

            link.send_packet(packet.into())?;
        } else {
            return Ok(());
        }
    }
}
