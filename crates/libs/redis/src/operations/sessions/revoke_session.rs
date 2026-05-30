use std::collections::HashSet;

use redis::AsyncTypedCommands;

use crate::{connection::Redis, error::RedisError};

pub async fn revoke_session(
  redis: &Redis,
  session_id: &str,
) -> Result<(), RedisError> {
  let mut conn = redis.conn().clone();

  let session_key = format!("session:{session_id}:tokens");

  let tokens: HashSet<String> = conn
    .smembers(&session_key)
    .await
    .map_err(RedisError::Redis)?;

  for token in &tokens {
    let auth_key = format!("auth:{token}");

    conn.del(&auth_key).await.map_err(RedisError::Redis)?;
  }

  conn.del(&session_key).await.map_err(RedisError::Redis)?;

  Ok(())
}
