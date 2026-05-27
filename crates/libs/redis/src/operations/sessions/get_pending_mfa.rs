use kestrel_common::models::session::PendingMfa;
use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

pub async fn get_pending_mfa(
    redis: &Redis,
    temp_token: &str,
) -> Result<Option<PendingMfa>, RedisError> {
    let key = format!("pending_mfa:{temp_token}");

    let mut conn = redis.conn().clone();

    let Some(value): Option<String> = conn.get(&key).await.map_err(RedisError::Redis)? else {
        return Ok(None);
    };

    let pending_mfa: PendingMfa =
        serde_json::from_str(&value).map_err(|e| RedisError::Other(e.to_string()))?;

    Ok(Some(pending_mfa))
}
