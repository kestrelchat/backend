use crate::connection::Database;
use crate::error::DatabaseError;

pub async fn revoke_session(
  db: &Database,
  session_id: &str,
) -> Result<(), DatabaseError> {
  sqlx::query(
    r#"
        DELETE FROM sessions
        WHERE id = $1
        "#,
  )
  .bind(session_id)
  .execute(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
