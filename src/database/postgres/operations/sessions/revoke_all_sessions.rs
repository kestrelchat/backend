use crate::database::postgres::connection::Database;
use crate::database::postgres::error::DatabaseError;

pub async fn revoke_all_sessions(
  postgres: &Database,
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
  .execute(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
