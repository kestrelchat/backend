ARG RUST_VERSION=1.96
ARG BUILD_MODE=debug

# Planner Stage
FROM rust:${RUST_VERSION} AS planner

WORKDIR /app

RUN cargo install cargo-chef

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src

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
COPY src ./src

RUN if [ "$BUILD_MODE" = "release" ]; then \
        cargo build --release -p dendryte ; \
    else \
        cargo build -p dendryte ; \
    fi

# Runtime stage
FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG BUILD_MODE

COPY --from=builder /app/target/${BUILD_MODE}/dendryte /usr/local/bin/dendryte

EXPOSE 5187

CMD ["dendryte"]
