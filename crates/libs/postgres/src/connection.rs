use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

use crate::error::DatabaseError;

#[derive(Clone)]
pub struct Database {
  pool: PgPool,
}

impl Database {
  pub async fn connect(url: &str) -> Result<Self, DatabaseError> {
    let pool = PgPoolOptions::new()
      .max_connections(20)
      .acquire_timeout(Duration::from_secs(5))
      .connect(url)
      .await
      .map_err(DatabaseError::from_sqlx)?;

    Ok(Self { pool })
  }

  pub fn pool(&self) -> &PgPool {
    &self.pool
  }

  pub async fn migrate(&self) -> Result<(), DatabaseError> {
    sqlx::migrate!("./migrations")
      .run(&self.pool)
      .await
      .map_err(DatabaseError::Migration)
  }
}
