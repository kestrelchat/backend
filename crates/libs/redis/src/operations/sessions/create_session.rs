use kestrel_common::{
  models::RedisSession,
  token::{Token, TokenType},
};
use redis::AsyncCommands;

use crate::{connection::Redis, error::RedisError};

const TTL_SECS: u64 = 20 * 60;

pub async fn create_session(
  redis: &Redis,
  session_id: &str,
  account_id: &str,
) -> Result<String, RedisError> {
  let auth_token = Token::generate(TokenType::Auth);

  let key = format!("auth:{auth_token}");

  let value = serde_json::to_string(&RedisSession {
    session_id: session_id.to_string(),
    account_id: account_id.to_string(),
  })
  .map_err(|e| RedisError::Other(e.to_string()))?;

  let mut conn = redis.conn().clone();

  conn
    .set_ex::<_, _, ()>(&key, &value, TTL_SECS)
    .await
    .map_err(RedisError::Redis)?;

  Ok(auth_token)
}
