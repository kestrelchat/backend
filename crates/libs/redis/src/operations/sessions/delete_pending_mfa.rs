use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

pub async fn delete_pending_mfa(redis: &Redis, temp_token: &str) -> Result<(), RedisError> {
    let key = format!("pending_mfa:{temp_token}");

    let mut conn = redis.conn().clone();
    let _: () = conn.del(&key).await.map_err(RedisError::Redis)?;

    Ok(())
}
