// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use redis::RedisError as InnerRedisError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("connection error: {0}")]
    Connection(InnerRedisError),

    #[error("client error: {0}")]
    Client(InnerRedisError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("timeout")]
    Timeout,

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("redis error: {0}")]
    Redis(InnerRedisError),

    #[error("unexpected response")]
    Unexpected,

    #[error("other error: {0}")]
    Other(String),
}
