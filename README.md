# Crazyradio server ![Rust](https://github.com/ataffanel/crazyradio-server/workflows/Rust/badge.svg)

Server exposing the Crazyflie link over a Crazyradio USB dongle as a JSONRPC API over ZMQ REP/REQ socket.

Functionalities:

- Scanning for Crazyflies
- Connecting Crazyflies using channel and address
- Connection optinally uses *Safelink* for lossless communication and exposes uplink/downling as a couple of ZMQ PUSH/PULL sockets transmitting raw CRTP packets
- Any number of connections is handled, they are scheduled in a round-robin manner.
- Handles multiple radio in parallel

Limitations:

- The datarate is fixed at 2Mbps
- No broadcast transmition
