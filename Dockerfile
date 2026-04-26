FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libi2c-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libi2c0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bme280-exporter /usr/local/bin/

ENTRYPOINT ["bme280-exporter"]
