use crate::connection::Database;
use crate::error::DatabaseError;

pub async fn revoke_all_sessions(
  db: &Database,
  user_id: &str,
  current_session: &str,
) -> Result<(), DatabaseError> {
  sqlx::query(
    r#"
        DELETE FROM sessions
        WHERE user_id = $1
        AND id != $2
        "#,
  )
  .bind(user_id)
  .bind(current_session)
  .execute(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
