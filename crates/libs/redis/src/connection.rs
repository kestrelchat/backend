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

use redis::Client;
use redis::aio::ConnectionManager;

use crate::error::RedisError;

#[derive(Clone)]
pub struct Redis {
    conn: ConnectionManager,
}

impl Redis {
    pub async fn connect(url: &str) -> Result<Self, RedisError> {
        let client = Client::open(url).map_err(RedisError::Client)?;

        let conn = client
            .get_connection_manager()
            .await
            .map_err(RedisError::Connection)?;

        Ok(Self { conn })
    }

    pub fn conn(&self) -> &ConnectionManager {
        &self.conn
    }
}
