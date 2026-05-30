use crate::connection::Database;
use crate::error::DatabaseError;

pub async fn revoke_all_sessions(
  db: &Database,
  user_id: &str,
) -> Result<(), DatabaseError> {
  sqlx::query(
    r#"
        DELETE FROM sessions
        WHERE user_id = $1
        "#,
  )
  .bind(user_id)
  .execute(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
