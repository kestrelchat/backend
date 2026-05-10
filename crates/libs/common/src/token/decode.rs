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

use crate::{
    token::{Token, error::TokenError, spec},
    utils::base32::base32_decode,
};

pub fn decode(input: &str) -> Result<Token, TokenError> {
    let bytes = base32_decode(input).map_err(|_| TokenError::InvalidEncoding)?;

    if bytes.len() != spec::TOTAL_BYTES {
        return Err(TokenError::InvalidLength);
    }

    let mut ts = [0u8; 8];
    ts[2..8].copy_from_slice(&bytes[0..6]);
    let timestamp = u64::from_be_bytes(ts);

    let version = bytes[6];

    if version != spec::VERSION {
        return Err(TokenError::UnsupportedVersion);
    }

    let token_type = bytes[7];

    let entropy: [u8; 16] = bytes[8..24]
        .try_into()
        .expect("slice is guaranteed to be 16 bytes");

    Ok(Token {
        timestamp,
        version,
        token_type,
        entropy,
    })
}
