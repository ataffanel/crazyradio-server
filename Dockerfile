FROM rust:1.44.1

WORKDIR /data

RUN apt-get update && apt-get install -y cmake

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ubuntu:focal

COPY --from=0 /data/target/release/crazyradio-server /usr/bin

ENTRYPOINT [ "crazyradio-server" ]
