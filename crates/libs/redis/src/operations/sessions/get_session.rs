use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};
use kestrel_common::models::RedisSession;

pub async fn get_session(
    redis: &Redis,
    auth_token: &str,
) -> Result<Option<RedisSession>, RedisError> {
    let key = format!("auth:{auth_token}");

    let mut conn = redis.conn().clone();

    let value: Option<String> = conn.get(&key).await.map_err(RedisError::Redis)?;

    let value = match value {
        Some(v) => v,
        None => return Ok(None),
    };

    let session: RedisSession =
        serde_json::from_str(&value).map_err(|e| RedisError::Other(e.to_string()))?;

    Ok(Some(session))
}
