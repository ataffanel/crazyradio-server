use crate::error::Result;
use crazyradio::{Channel, Crazyradio};
use crossbeam_channel::{Receiver, Sender};
use std::convert::Into;

pub enum RadioCommand {
    SendPacket {
        client: Sender<Result<SendPacketResult>>,
        channel: Channel,
        payload: Vec<u8>,
    },
    Scan {
        client: Sender<Result<ScanResult>>,
        start: Channel,
        stop: Channel,
        payload: Vec<u8>,
    },
}

pub struct SendPacketResult {
    pub acked: bool,
    pub payload: Vec<u8>,
}
pub struct ScanResult {
    pub found: Vec<u8>,
}

fn scan(
    crazyradio: &mut Crazyradio,
    start: Channel,
    stop: Channel,
    payload: Vec<u8>,
) -> Result<ScanResult> {
    let found = crazyradio.scan_channels(start, stop, &payload)?;

    let found = found.into_iter().map(|ch| ch.into()).collect();

    return Ok(ScanResult { found });
}

fn send_packet(
    crazyradio: &mut Crazyradio,
    channel: Channel,
    payload: Vec<u8>,
) -> Result<SendPacketResult> {
    let mut ack_data = Vec::new();
    ack_data.resize(32, 0);
    crazyradio.set_channel(channel)?;

    let ack = crazyradio.send_packet(&payload, &mut ack_data)?;
    ack_data.resize(ack.length, 0);

    Ok(SendPacketResult {
        acked: ack.received,
        payload: ack_data,
    })
}

pub fn radio_loop(crazyradio: Crazyradio, radio_cmd: Receiver<RadioCommand>) {
    let mut crazyradio = crazyradio;
    for command in radio_cmd {
        match command {
            RadioCommand::Scan {
                client,
                start,
                stop,
                payload,
            } => {
                let res = scan(&mut crazyradio, start, stop, payload);
                client.send(res).unwrap();
            }
            RadioCommand::SendPacket {
                client,
                channel,
                payload,
            } => {
                let res = send_packet(&mut crazyradio, channel, payload);
                client.send(res).unwrap();
            }
        }
    }
}
