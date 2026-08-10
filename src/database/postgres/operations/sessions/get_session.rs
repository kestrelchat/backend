use crate::models::Session;
use chrono::Utc;
use ulid::Ulid;

use crate::{
  adapters::crypto::hasher,
  database::postgres::{connection::Database, error::DatabaseError},
  models::token::{Token, TokenType},
};

pub struct PostgresCreatedSession {
  pub session: Session,
  pub refresh_token: String,
}

pub struct SessionMetadata {
  pub ip_address: Option<std::net::IpAddr>,
  pub country: Option<String>,
  pub region: Option<String>,
  pub city: Option<String>,
  pub user_agent: Option<String>,
  pub operating_system: Option<String>,
  pub platform: Option<String>,
}

pub async fn get_session(
  postgres: &Database,
  refresh_token: &str,
) -> Result<Vec<Session>, DatabaseError> {
  let refresh_token_hash = hasher::hash(refresh_token.as_bytes());
  let sessions = sqlx::query_as::<_, Session>(
    r#"
        SELECT *
        FROM sessions
        WHERE refresh_token = $1
        ORDER BY created_at DESC
        "#,
  )
  .bind(refresh_token_hash)
  .fetch_all(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(sessions)
}
