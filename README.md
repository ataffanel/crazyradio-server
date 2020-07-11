# Crazyradio server ![Rust](https://github.com/ataffanel/crazyradio-server/workflows/Rust/badge.svg)

Server exposing a Crazyradio USB dongle as a JSONRPC API over ZMQ REP/REQ socket.

Functionalities:

- Single packet send/ack receive
- Scanning a range of channels
- Connecting Crazyflies using channel and address
- Connection uses *Safelink* for lossless communication and exposes uplink/downling as a couple of ZMQ PUSH/PULL sockets transmitting raw CRTP packets
- Any number of connections is handled, they are scheduled in a round-robin manner.

Not (yet) implemented:

- The datarate is fixed at 2Mbps
- No broadcast transmition
- Currently only opens the first found Crazyradio, no command line arguments implemented yet.
