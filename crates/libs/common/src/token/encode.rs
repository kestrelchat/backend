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
    token::{spec, types::Token},
    utils::base32::base32_encode,
};

pub fn encode(token: &Token) -> String {
    let mut bytes = Vec::with_capacity(spec::TOTAL_BYTES);

    let ts = token.timestamp.to_be_bytes();
    bytes.extend_from_slice(&ts[2..8]);

    bytes.push(token.version);

    bytes.push(token.token_type as u8);

    bytes.extend_from_slice(&token.entropy);

    base32_encode(&bytes)
}
