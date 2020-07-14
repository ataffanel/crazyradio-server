// Connection handling code
use crate::error::{Error, Result};
use crate::radio_thread::RadioThread;
use crazyradio::Channel;
use crossbeam_utils::sync::{ShardedLock, WaitGroup};
use log::{debug, info, warn};
use rand;
use rand::Rng;
use std::sync::Arc;
use std::sync::RwLock;
use std::time;

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected(String),
}

pub struct Connection {
    status: Arc<RwLock<ConnectionStatus>>,
    disconnect: Arc<ShardedLock<bool>>,
    thread: std::thread::JoinHandle<()>,
    push_port: u16,
    pull_port: u16,
}

fn bind_zmq_on_random_port(socket: &zmq::Socket) -> Result<u16> {
    let n_try = 10;

    for _ in 0..n_try {
        let port = rand::thread_rng().gen_range(49152, 65535);

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
    pub fn new(radio: RadioThread, channel: Channel, address: [u8; 5]) -> Result<Connection> {
        let status = Arc::new(RwLock::new(ConnectionStatus::Connecting));
        let disconnect = Arc::new(ShardedLock::new(false));

        // Create two ZMQ? socket for sending and receiving raw CRTP packets
        let context = zmq::Context::new();

        let socket_push = context.socket(zmq::PUSH).unwrap();
        let socket_pull = context.socket(zmq::PULL).unwrap();

        let push_port = bind_zmq_on_random_port(&socket_push)?;
        let pull_port = bind_zmq_on_random_port(&socket_pull)?;

        let connection_initialized = WaitGroup::new();

        let ci = connection_initialized.clone();
        let mut thread = ConnectionThread::new(
            radio,
            status.clone(),
            disconnect.clone(),
            socket_push,
            socket_pull,
            channel,
            address,
        );
        let thread = std::thread::spawn(move || match thread.run(ci) {
            Err(e) => thread.update_status(ConnectionStatus::Disconnected(format!(
                "Connection error: {}",
                e
            ))),
            _ => {}
        });

        // Wait for, either, the connection being established or failed initialization
        connection_initialized.wait();

        Ok(Connection {
            status,
            disconnect,
            thread,
            push_port,
            pull_port,
        })
    }

    pub fn status(&self) -> ConnectionStatus {
        self.status.read().unwrap().clone()
    }

    pub fn disconnect(self) {
        *self.disconnect.write().unwrap() = true;
        debug!("Closing the connection!");
        self.thread.join().unwrap();
    }

    pub fn get_zmq_ports(&self) -> (u16, u16) {
        (self.pull_port, self.push_port)
    }
}

struct ConnectionThread {
    radio: RadioThread,
    status: Arc<RwLock<ConnectionStatus>>,
    disconnect: Arc<ShardedLock<bool>>,
    safelink_up_ctr: u8,
    safelink_down_ctr: u8,
    socket_push: zmq::Socket,
    socket_pull: zmq::Socket,
    channel: Channel,
    address: [u8; 5],
}

impl ConnectionThread {
    fn new(
        radio: RadioThread,
        status: Arc<RwLock<ConnectionStatus>>,
        disconnect: Arc<ShardedLock<bool>>,
        socket_push: zmq::Socket,
        socket_pull: zmq::Socket,
        channel: Channel,
        address: [u8; 5],
    ) -> Self {
        ConnectionThread {
            radio,
            status,
            disconnect,
            safelink_up_ctr: 0,
            safelink_down_ctr: 0,
            socket_push,
            socket_pull,
            channel,
            address,
        }
    }

    fn update_status(&self, new_status: ConnectionStatus) {
        debug!("New status: {:?}", &new_status);
        let mut status = self.status.write().unwrap();
        *status = new_status;
    }

    fn enable_safelink(&mut self) -> Result<bool> {
        // Tying 10 times to reset safelink
        for _ in 0..10 {
            let (ack, payload) =
                self.radio
                    .send_packet(self.channel, self.address, vec![0xff, 0x05, 0x01])?;

            if ack.received && payload == [0xff, 0x05, 0x01] {
                self.safelink_down_ctr = 0;
                self.safelink_up_ctr = 0;

                // Safelink enabled!
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn send_packet_safe(&mut self, packet: Vec<u8>) -> Result<(crazyradio::Ack, Vec<u8>)> {
        let mut packet = packet;
        packet[0] &= 0xF3;
        packet[0] |= (self.safelink_up_ctr << 3) | (self.safelink_down_ctr << 2);

        let (ack, ack_payload) = self.radio.send_packet(self.channel, self.address, packet)?;

        if ack.received && ack_payload.len() > 0 {
            let received_down_ctr = (ack_payload[0] & 0x04) >> 2;
            if received_down_ctr == self.safelink_down_ctr {
                self.safelink_down_ctr = 1 - self.safelink_down_ctr;
            }
        }

        if ack.received {
            self.safelink_up_ctr = 1 - self.safelink_up_ctr;
        }

        Ok((ack, ack_payload))
    }

    fn run(&mut self, connection_initialized: WaitGroup) -> Result<()> {
        info!("Connecting to {:?}/{:X?} ...", self.channel, self.address);

        // Try to initialize safelink
        // This server only supports safelink, if it cannot be enabled
        // the Crazyflie is deemed not connectable
        if self.enable_safelink()? == false {
            self.update_status(ConnectionStatus::Disconnected(
                "Cannot initialize connection".to_string(),
            ));
            warn!(
                "Cannot initialize connection with {:?}/{:X?}",
                self.channel, self.address
            );
            return Ok(());
        }

        // Safelink is initialized, we are connected!
        self.update_status(ConnectionStatus::Connected);
        drop(connection_initialized);

        info!("Connected to {:?}/{:X?}", self.channel, self.address);

        // Wait for push connection to be active?
        self.socket_push.send(vec![0xff], 0)?;
        self.socket_push.set_sndtimeo(10).unwrap();

        debug!("Zmq PUSH socket connected, starting communication loop");

        // Communication loop ...
        let mut last_pk_time = time::Instant::now();
        let mut relax_timeout_ms = 10;
        let mut packet = vec![0xff];
        let mut needs_resend = false;
        while last_pk_time.elapsed() < time::Duration::from_millis(1000) {
            if !needs_resend {
                packet = match self.socket_pull.poll(zmq::POLLIN, relax_timeout_ms)? {
                    0 => vec![0xff], // NULL packet
                    _ => self.socket_pull.recv_bytes(0)?,
                };
            }

            let (ack, mut ack_payload) = self.send_packet_safe(packet.clone())?;

            if ack.received {
                last_pk_time = time::Instant::now();
                needs_resend = false;

                // Add some relaxation time if the Crazyflie has nothing to send back
                // We may want to be a bit more clever there (ie. increasing the time by
                // small increment instead of this all-or-nothing aproach)
                if ack_payload.len() > 0 && (ack_payload[0] & 0xF3) != 0xF3 {
                    ack_payload[0] &= 0xF3;
                    self.socket_push.send(&ack_payload, 0)?;
                    relax_timeout_ms = 0;
                } else {
                    // If no packet received, relax packet pulling
                    relax_timeout_ms = 10;
                }
            } else {
                debug!("Lost packet!");
                needs_resend = true;
            }

            if *self.disconnect.read().unwrap() {
                self.update_status(ConnectionStatus::Disconnected(
                    "Disconnect requested".to_string(),
                ));
                info!(
                    "Closing connection to {:?}/{:X?} as requested",
                    self.channel, self.address
                );
                return Ok(());
            }
        }

        self.update_status(ConnectionStatus::Disconnected(
            "Connection timeout".to_string(),
        ));

        warn!(
            "Connection to {:?}/{:X?} lost (timeout)",
            self.channel, self.address
        );

        Ok(())
    }
}
