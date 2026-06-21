use redis::Client;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};

use crate::error::RedisError;

#[derive(Clone)]
pub struct Redis {
  conn: ConnectionManager,
}

impl Redis {
  /// Returns the connection manager configuration based on the current environment.
  ///
  /// It's set to more forgiving timeouts for testing, but defaults to reasonable values in production.
  fn config() -> ConnectionManagerConfig {
    #[cfg(test)]
    {
      use std::time::Duration;
      ConnectionManagerConfig::new()
        .set_connection_timeout(Some(Duration::from_secs(5)))
        .set_response_timeout(Some(Duration::from_secs(2)))
    }
    #[cfg(not(test))]
    {
      ConnectionManagerConfig::new()
    }
  }

  pub async fn connect(url: &str) -> Result<Self, RedisError> {
    let client = Client::open(url).map_err(RedisError::Client)?;

    let conn = client
      .get_connection_manager_with_config(Self::config())
      .await
      .map_err(RedisError::Connection)?;

    Ok(Self { conn })
  }

  pub fn conn(&self) -> &ConnectionManager {
    &self.conn
  }
}
