use kestrel_common::{
    models::session::PendingMfa,
    token::{Token, TokenType},
};
use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

const TTL_SECS: u64 = 20 * 60;

pub async fn create_pending_mfa(
    redis: &Redis,
    pending_mfa: PendingMfa,
) -> Result<String, RedisError> {
    let temp_token = Token::generate(TokenType::PendingMfa);
    let key = format!("pending_mfa:{temp_token}");

    let value =
        serde_json::to_string(&pending_mfa).map_err(|e| RedisError::Other(e.to_string()))?;

    let mut conn = redis.conn().clone();

    conn.set_ex::<_, _, ()>(&key, value, TTL_SECS)
        .await
        .map_err(RedisError::Redis)?;

    Ok(temp_token)
}
