FROM rust:1.94 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

ENV KESTREL_CONFIG=/var/kestrel/conf.toml
COPY kestrel.toml $KESTREL_CONFIG

RUN cargo build --release -p gateway

FROM debian:trixie-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway

EXPOSE 5180

CMD ["gateway"]
