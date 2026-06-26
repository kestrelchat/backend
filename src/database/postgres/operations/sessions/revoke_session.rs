use crate::database::postgres::connection::Database;
use crate::database::postgres::error::DatabaseError;

pub async fn revoke_session(
  postgres: &Database,
  session_id: &str,
) -> Result<(), DatabaseError> {
  sqlx::query(
    r#"
        DELETE FROM sessions
        WHERE id = $1
        "#,
  )
  .bind(session_id)
  .execute(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
