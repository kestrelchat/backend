/*
 * Kestrel - a modern instant-messaging service written in Rust
 * Copyright (C) 2026 Kestrel Chat
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(try_from = "u8")]
pub enum OpCode {
  Dispatch,
  Heartbeat,
  Identify,
}

impl TryFrom<u8> for OpCode {
  type Error = String;
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(OpCode::Dispatch),
      1 => Ok(OpCode::Heartbeat),
      2 => Ok(OpCode::Identify),
      other => Err(format!("unknown opcode: {other}")),
      // Everything Bagels. - Stribes
    }
  }
}
