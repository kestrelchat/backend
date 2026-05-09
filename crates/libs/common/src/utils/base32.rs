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

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn base32_encode(data: &[u8]) -> String {
    let mut output = String::new();

    let mut buffer = 0u64;
    let mut bits_left = 0;

    for &byte in data {
        buffer <<= 8;
        buffer |= byte as u64;
        bits_left += 8;

        while bits_left >= 5 {
            let index = ((buffer >> (bits_left - 5)) & 0x1F) as usize;
            output.push(ALPHABET[index] as char);
            bits_left -= 5;
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        output.push(ALPHABET[index] as char);
    }

    output
}

pub fn base32_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let mut buffer: u64 = 0;
    let mut bits_left: u8 = 0;

    let mut output: Vec<u8> = Vec::new();

    for c in input.bytes() {
        let mut value: Option<u8> = None;

        for (i, &a) in ALPHABET.iter().enumerate() {
            if a == c {
                value = Some(i as u8);
                break;
            }
        }

        let value = value.ok_or("invalid base32 character")?;

        buffer <<= 5;
        buffer |= value as u64;
        bits_left += 5;

        while bits_left >= 8 {
            let byte = (buffer >> (bits_left - 8)) & 0xFF;
            output.push(byte as u8);
            bits_left -= 8;
        }
    }

    Ok(output)
}
