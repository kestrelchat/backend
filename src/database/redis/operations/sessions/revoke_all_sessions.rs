use redis::AsyncTypedCommands;

use crate::database::redis::{connection::Redis, error::RedisError};

pub async fn revoke_all_sessions(
  redis: &Redis,
  account_id: &str,
  current_token: &str,
) -> Result<(), RedisError> {
  let mut conn = redis.conn().clone();

  let user_key = format!("user:{}:tokens", account_id);

  let tokens: std::collections::HashSet<String> = conn
    .smembers(&user_key)
    .await
    .map_err(RedisError::Redis)?
    .into_iter()
    .collect();

  for token in &tokens {
    if token == current_token {
      continue;
    }

    let auth_key = format!("auth:{token}");
    conn.del(&auth_key).await.map_err(RedisError::Redis)?;
  }

  conn.del(&user_key).await.map_err(RedisError::Redis)?;

  Ok(())
}
