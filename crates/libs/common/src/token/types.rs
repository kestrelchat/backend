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

use rand::TryRng;
use rand::rngs::SysRng;

use crate::token::encode::encode;
use crate::token::spec::VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TokenType {
    Refresh = 1,
    Auth = 2,
}

impl TryFrom<u8> for TokenType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Refresh),
            2 => Ok(Self::Auth),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Token {
    pub version: u8,
    pub timestamp: u64,
    pub token_type: TokenType,
    pub entropy: [u8; 16],
}

impl Token {
    fn new(token_type: TokenType) -> Self {
        Self {
            version: VERSION,
            timestamp: current_time_millis(),
            token_type,
            entropy: secure_random_16(),
        }
    }

    pub fn generate(token_type: TokenType) -> String {
        let token = Self::new(token_type);
        encode(&token)
    }

    pub fn is_expired(&self, ttl_ms: u64) -> bool {
        let now = current_time_millis();
        now.saturating_sub(self.timestamp) > ttl_ms
    }
}

fn current_time_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock went backwards")
        .as_millis() as u64
}

fn secure_random_16() -> [u8; 16] {
    let mut buf = [0u8; 16];
    SysRng.try_fill_bytes(&mut buf).expect("OS RNG failed");
    buf
}
