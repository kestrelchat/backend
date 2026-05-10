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

pub const VERSION: u8 = 1;

pub const TIMESTAMP_BYTES: usize = 6;
pub const VERSION_BYTES: usize = 1;
pub const TYPE_BYTES: usize = 1;
pub const ENTROPY_BYTES: usize = 16;

pub const TOTAL_BYTES: usize = TIMESTAMP_BYTES + VERSION_BYTES + TYPE_BYTES + ENTROPY_BYTES;
pub const ENCODED_LENGTH: usize = 39;
