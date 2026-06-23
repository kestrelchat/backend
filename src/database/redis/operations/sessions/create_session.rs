use crate::database::redis::connection::Redis;
use crate::database::redis::error::RedisError;
use crate::models::RedisSession;
use crate::models::token::{Token, TokenType};
use redis::{AsyncCommands, aio::ConnectionManager};

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

  // user_id -> tokens index
  let user_key = format!("user:{}:tokens", account_id);

  index_token(&mut conn, &user_key, &auth_token).await?;

  // session_id -> tokens index
  let session_key = format!("session:{}:tokens", session_id);

  index_token(&mut conn, &session_key, &auth_token).await?;

  conn
    .set_ex::<_, _, ()>(&key, &value, TTL_SECS)
    .await
    .map_err(RedisError::Redis)?;

  Ok(auth_token)
}

pub async fn index_token(
  conn: &mut ConnectionManager,
  index_key: &str,
  token: &str,
) -> Result<(), RedisError> {
  conn
    .sadd::<_, _, ()>(index_key, token)
    .await
    .map_err(RedisError::Redis)?;

  conn
    .expire::<_, ()>(index_key, TTL_SECS as i64)
    .await
    .map_err(RedisError::Redis)?;

  Ok(())
}
