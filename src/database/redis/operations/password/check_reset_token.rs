use redis::AsyncCommands;

use crate::database::redis::{connection::Redis, error::RedisError};

pub async fn check_reset_token(
  redis: &Redis,
  token: &str,
) -> Result<String, RedisError> {
  let key = format!("reset:{token}");

  let mut conn = redis.conn().clone();

  let account_id: Option<String> =
    conn.get(&key).await.map_err(RedisError::Redis)?;

  let account_id = match account_id {
    Some(id) => id,
    None => return Err(RedisError::NotFound),
  };

  Ok(account_id)
}
