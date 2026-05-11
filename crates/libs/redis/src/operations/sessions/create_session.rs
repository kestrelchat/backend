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

use kestrel_common::{
    models::RedisSession,
    token::{Token, TokenType},
};
use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

const TTL_SECS: u64 = 20 * 60;

pub async fn create_session(
    redis: &Redis,
    session_id: &str,
    user_id: &str,
) -> Result<String, RedisError> {
    let auth_token = Token::generate(TokenType::Auth);

    let key = format!("auth:{auth_token}");

    let value = serde_json::to_string(&RedisSession {
        session_id: session_id.to_string(),
        user_id: user_id.to_string(),
    })
    .map_err(|e| RedisError::Other(e.to_string()))?;

    let mut conn = redis.conn().clone();

    conn.set_ex::<_, _, ()>(&key, &value, TTL_SECS)
        .await
        .map_err(RedisError::Redis)?;

    Ok(auth_token)
}
