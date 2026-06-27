<p align="center">
    <img src="https://github.com/kestrelchat/kestrelchat/blob/main/branding/png/Kestrel__Repo-Header.png?raw=true" alt="Kestrel Banner">
</p>
<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.96%2B-6e6ade?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/github/license/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/stars/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/forks/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/last-commit/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/commit-activity/m/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/contributors/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/issues/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/issues-pr/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://img.shields.io/github/languages/code-size/kestrelchat/server?style=for-the-badge&color=6e6ade" />
  <img src="https://www.aschey.tech/tokei/github.com/kestrelchat/server?style=for-the-badge&color=6e6ade&language=Rust,Dockerfile" />
  <a href="https://discord.gg/T8rAX8DmNS">
    <img src="https://img.shields.io/discord/1453177758233661706?style=for-the-badge&logo=discord&logoColor=white&color=6e6ade" />
  </a>
</p>

# About

Dendryte is the official open-source server implementation for Kestrel, an end-to-end communication platform. It's built and maintained by the Kestrel team alongside a growing community of contributors who help shape its development.

# Contributing
## Prerequisites

- Rust 1.94+
- Any IDE with Rust support (I recommend Zed, with AI turned off, please.)
- Docker with Docker Compose (recommended)

## Configuration
Kestrel uses a shared TOML configuration file.

To get started, copy the example file `dendryte.example.toml` to `dendryte.toml`

Everything should be preconfigured for hosting on a single machine, through Docker.

## Running Dendryte

### For Development

#### With Docker

> Recommended for most developers.

```bash
docker compose up --build
```

#### Without Docker

Set your config path:
```bash
export DENDRYTE_CONFIG=path/to/dendryte.toml
```

Then run the server:
```bash
cargo run
```

Any external services (databases, etc.) must be configured in `dendryte.toml`.

### For Production

Pre-built Docker images are not yet available. You can build a release image yourself with `docker build .`. Production deployment docs will follow once the project is production-ready.

Please note that Dendryte is still under active development and has not yet been recommended for production use.

# External Libraries

Dendryte is built on top of the following open-source Rust libraries:

- [argon2](https://github.com/RustCrypto/password-hashes/tree/master/argon2)
- [base32](https://github.com/nicowilliams/rust-base32)
- [base64](https://github.com/marshallpierce/rust-base64)
- [blake3](https://github.com/BLAKE3-team/BLAKE3)
- [chacha20poly1305](https://github.com/RustCrypto/AEADs/tree/master/chacha20poly1305)
- [chrono](https://github.com/chronotope/chrono)
- [hcaptcha](https://github.com/juliankrispel/hcaptcha-rust)
- [rand](https://github.com/rust-random/rand)
- [redis](https://github.com/redis-rs/redis-rs)
- [reqwest](https://github.com/seanmonstar/reqwest)
- [rocket](https://github.com/rwf2/Rocket)
- [rocket_okapi](https://github.com/GREsau/rocket_okapi)
- [rustc-hash](https://github.com/rust-lang/rustc-hash)
- [schemars](https://github.com/GREsau/schemars)
- [serde](https://github.com/serde-rs/serde)
- [serde_json](https://github.com/serde-rs/json)
- [sqlx](https://github.com/launchbadge/sqlx)
- [testcontainers-modules](https://github.com/testcontainers/testcontainers-rs)
- [thiserror](https://github.com/dtolnay/thiserror)
- [toml](https://github.com/toml-rs/toml)
- [totp-rs](https://github.com/constantoine/totp-rs)
- [uaparser](https://github.com/ua-parser/uap-rust)
- [ulid](https://github.com/ulid/spec)
- [zeroize](https://github.com/RustCrypto/utils/tree/master/zeroize)
