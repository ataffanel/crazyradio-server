# Crazyradio server ![Rust](https://github.com/ataffanel/crazyradio-server/workflows/Rust/badge.svg)

Server exposing a Crazyradio USB dongle as a JSONRPC API over ZMQ REP/REQ socket.

Functionalities:

- Single packet send/ack receive
- Scanning a range of channels
- Connecting Crazyflies using channel and address
- Connection uses *Safelink* for lossless communication and exposes uplink/downling as a couple of ZMQ PUSH/PULL sockets transmitting raw CRTP packets
- Any number of connections is handled, they are scheduled in a round-robin manner.

Limitations:

- The datarate is fixed at 2Mbps
- No broadcast transmition
- Serves only one radio, if many radio needs to be served the server can be run multiple times serving each radios on different ports
