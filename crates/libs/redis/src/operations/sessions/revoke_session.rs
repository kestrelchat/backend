use redis::AsyncTypedCommands;

use crate::{connection::Redis, error::RedisError};

pub async fn revoke_session(
  redis: &Redis,
  auth_token: &str,
) -> Result<(), RedisError> {
  let key = format!("auth:{auth_token}");

  let mut conn = redis.conn().clone();

  conn.del(&key).await.map_err(RedisError::Redis)?;

  Ok(())
}
