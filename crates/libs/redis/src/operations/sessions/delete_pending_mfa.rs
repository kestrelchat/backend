use kestrel_common::utils::hasher::hash;
use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

pub async fn delete_pending_mfa(redis: &Redis, temp_token: &str) -> Result<(), RedisError> {
    let token_hash = hash(temp_token.as_bytes());
    let key = format!("pending_mfa:{token_hash}");

    let mut conn = redis.conn().clone();
    let _: () = conn.del(&key).await.map_err(RedisError::Redis)?;

    Ok(())
}
