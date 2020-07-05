FROM rust:1.44.1-alpine3.12 AS builder

WORKDIR /data

RUN apk add cmake make g++ linux-headers
RUN rustup target add x86_64-unknown-linux-gnu

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM alpine:3.12

COPY --from=builder /data/target/x86_64-unknown-linux-gnu/release/crazyradio-server /usr/bin

RUN apk add libstdc++ libgcc

ENTRYPOINT [ "crazyradio-server" ]
