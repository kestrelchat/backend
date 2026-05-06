ARG RUST_VERSION=1.94
ARG BUILD_MODE=debug

# Planner Stage
FROM rust:${RUST_VERSION} AS planner

WORKDIR /app

RUN cargo install cargo-chef

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo chef prepare --recipe-path recipe.json

# Builder Stage
FROM rust:${RUST_VERSION} AS builder

WORKDIR /app

RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json

ARG BUILD_MODE

RUN if [ "$BUILD_MODE" = "release" ]; then \
        cargo chef cook --release --recipe-path recipe.json ; \
    else \
        cargo chef cook --recipe-path recipe.json ; \
    fi

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN if [ "$BUILD_MODE" = "release" ]; then \
        cargo build --release -p kestrel_api ; \
    else \
        cargo build -p kestrel_api ; \
    fi

# Runtime stage
FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG BUILD_MODE

COPY --from=builder /app/target/${BUILD_MODE}/kestrel_api /usr/local/bin/kestrel_api

EXPOSE 5180

CMD ["kestrel_api"]
