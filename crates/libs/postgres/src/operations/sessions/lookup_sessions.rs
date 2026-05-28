use crate::{connection::Database, error::DatabaseError};
use kestrel_common::models::Session;

pub async fn lookup_sessions(
  db: &Database,
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
  .fetch_all(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(sessions)
}
