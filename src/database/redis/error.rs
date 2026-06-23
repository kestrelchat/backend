use redis::RedisError as InnerRedisError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedisError {
  #[error("not found")]
  NotFound,

  #[error("connection error: {0}")]
  Connection(InnerRedisError),

  #[error("client error: {0}")]
  Client(InnerRedisError),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("timeout")]
  Timeout,

  #[error("protocol error: {0}")]
  Protocol(String),

  #[error("redis error: {0}")]
  Redis(InnerRedisError),

  #[error("unexpected response")]
  Unexpected,

  #[error("other error: {0}")]
  Other(String),
}
