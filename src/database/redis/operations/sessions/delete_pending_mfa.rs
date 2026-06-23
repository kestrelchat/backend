use crate::{
  adapters::crypto::hasher::hash,
  database::redis::{connection::Redis, error::RedisError},
};
use redis::AsyncCommands;

pub async fn delete_pending_mfa(
  redis: &Redis,
  temp_token: &str,
) -> Result<(), RedisError> {
  let token_hash = hash(temp_token.as_bytes());
  let key = format!("pending_mfa:{token_hash}");

  let mut conn = redis.conn().clone();
  let _: () = conn.del(&key).await.map_err(RedisError::Redis)?;

  Ok(())
}
