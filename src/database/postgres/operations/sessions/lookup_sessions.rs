use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::Session;

pub async fn lookup_sessions(
  postgres: &Database,
  user_id: &str,
) -> Result<Vec<Session>, DatabaseError> {
  let sessions = sqlx::query_as::<_, Session>(
    r#"
        SELECT *
        FROM sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
  )
  .bind(user_id)
  .fetch_all(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(sessions)
}
