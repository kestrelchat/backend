use kestrel_common::{models::session::PendingMfa, utils::hasher::hash};
use redis::AsyncCommands;

use crate::{
    connection::Redis, error::RedisError, utils::protected_pending_mfa::ProtectedPendingMfa,
};

pub async fn get_pending_mfa(
    redis: &Redis,
    temp_token: &str,
) -> Result<Option<PendingMfa>, RedisError> {
    let token_hash = hash(temp_token.as_bytes());
    let key = format!("pending_mfa:{token_hash}");

    let mut conn = redis.conn().clone();

    let Some(value): Option<String> = conn.get(&key).await.map_err(RedisError::Redis)? else {
        return Ok(None);
    };

    let protected_pending_mfa: ProtectedPendingMfa =
        serde_json::from_str(&value).map_err(|e| RedisError::Other(e.to_string()))?;

    let pending_mfa = protected_pending_mfa
        .decrypt(temp_token)
        .map_err(|_| RedisError::Other("decryption failed".to_string()))?;

    Ok(Some(pending_mfa))
}
