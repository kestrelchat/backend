<p align="center">
    <img src="https://github.com/kestrelchat/kestrelchat/blob/main/branding/png/Kestrel__Repo-Header.png?raw=true" alt="Kestrel Banner">
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.94%2B-6e6ade?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/github/license/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/stars/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/forks/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/last-commit/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/commit-activity/m/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/contributors/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/issues/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/issues-pr/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/languages/code-size/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://www.aschey.tech/tokei/github.com/kestrelchat/server?style=for-the-badge&color=6e6ade&language=Rust,Dockerfile,Python" />
  <a href="https://discord.gg/T8rAX8DmNS">
    <img src="https://img.shields.io/discord/1453177758233661706?style=for-the-badge&logo=discord&logoColor=white&color=6e6ade" />
  </a>
</p>

# About

Kestrel is a free, open-source modern instant-messaging platform.

It is designed to be self-hostable, extensible, and lightweight.

This repository contains only the backend implementation of the system.

# Contributing
## Prerequisites

- Rust 1.94+
- Any IDE with Rust support (I recommend Zed, with AI turned off, please.)
- Docker with Docker Compose (recommended)

## Configuration
Kestrel uses a shared TOML configuration file.

To get started, copy the example file `kestrel.example.toml` to `kestrel.toml`

Everything should be preconfigured for hosting on a single machine, through docker.

# Structure

## Services
- **gateway** - WebSocket handling
- **api** - REST API backend

## Library
- **config** - Shared configuration library for Kestrel

## Running

### With Docker

> Recommended for most users.

To run the main backend (api, gateway, etc.) without development dependencies (e.g. Postgres):
```bash
BUILD_MODE=release docker compose --profile prod up --build --build-arg BUILD_MODE=release
```

To run all development dependencies (e.g. Postgres):
```bash
BUILD_MODE=debug docker compose --profile dev up --build
```

To run specific services:
```bash
BUILD_MODE=debug docker compose up --build api --build-arg BUILD_MODE=debug
```

#### Build Mode
``BUILD_MODE`` controls whether Rust is compiled in debug (fast, unoptimized) or release (optimized for production) mode inside Docker images.

Using ``BUILD_MODE=debug`` is recommended during development.

### Without Docker

Set your config path:
```bash
export KESTREL_CONFIG=path/to/kestrel.toml
```

Then run the services manually with:
```bash
cargo run -p <service>
```

Any external services (databases, cdns, etc.) must be configured in ``kestrel.toml``.

# External Libraries

Kestrel Backend is built on top of the following open-source Rust libraries:

- [argon2](https://github.com/RustCrypto/password-hashes/tree/master/argon2)
- [async-trait](https://github.com/dtolnay/async-trait)
- [chrono](https://github.com/chronotope/chrono)
- [once_cell](https://github.com/matklad/once_cell)
- [rand](https://github.com/rust-random/rand)
- [redis](https://github.com/redis-rs/redis-rs)
- [reqwest](https://github.com/seanmonstar/reqwest)
- [rocket](https://github.com/rwf2/Rocket)
- [rocket_okapi](https://github.com/GREsau/rocket_okapi)
- [rocket_ws](https://github.com/SergioBenitez/rocket_ws)
- [schemars](https://github.com/GREsau/schemars)
- [serde](https://github.com/serde-rs/serde)
- [serde_json](https://github.com/serde-rs/json)
- [sqlx](https://github.com/launchbadge/sqlx)
- [thiserror](https://github.com/dtolnay/thiserror)
- [tokio](https://github.com/tokio-rs/tokio)
- [toml](https://github.com/toml-rs/toml)
- [uaparser](https://github.com/ua-parser/uap-rust)
- [ulid](https://github.com/ulid/spec)
