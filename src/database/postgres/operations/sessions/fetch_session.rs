use crate::{
  database::postgres::{connection::Database, error::DatabaseError},
  models::Session,
};

pub async fn lookup_session(
  postgres: &Database,
  user_id: &str,
  id: &str,
) -> Result<Session, DatabaseError> {
  let session = sqlx::query_as::<_, Session>(
    r#"
        SELECT *
        FROM sessions
        WHERE user_id = $1 AND id = $2
        "#,
  )
  .bind(user_id)
  .bind(id)
  .fetch_one(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(session)
}
