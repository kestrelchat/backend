use redis::AsyncCommands;

use crate::database::redis::{connection::Redis, error::RedisError};
use crate::models::token::{Token, TokenType};

pub async fn create_reset_token(
  redis: &Redis,
  account_id: &str,
) -> Result<String, RedisError> {
  let reset_token = Token::generate(TokenType::PasswordReset);
  let key = format!("reset:{reset_token}");

  let value = account_id;

  let mut conn = redis.conn().clone();

  conn
    .set_ex::<_, _, ()>(&key, &value, 15 * 60)
    .await
    .map_err(RedisError::Redis)?;

  Ok(reset_token)
}
