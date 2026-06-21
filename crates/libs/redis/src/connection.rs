use redis::Client;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};

use crate::error::RedisError;

#[derive(Clone)]
pub struct Redis {
  conn: ConnectionManager,
}

impl Redis {
  pub async fn connect(url: &str) -> Result<Self, RedisError> {
    let client = Client::open(url).map_err(RedisError::Client)?;

    let conn = client
      .get_connection_manager_with_config(
        ConnectionManagerConfig::new().set_response_timeout(None),
      )
      .await
      .map_err(RedisError::Connection)?;

    Ok(Self { conn })
  }

  pub fn conn(&self) -> &ConnectionManager {
    &self.conn
  }
}
