use crate::models::user::profile::ProfileSummary;

use crate::database::postgres::{connection::Database, error::DatabaseError};

pub async fn batch_get_user_summaries(
  postgres: &Database,
  user_ids: &[String],
) -> Result<Vec<ProfileSummary>, DatabaseError> {
  if user_ids.is_empty() {
    return Ok(vec![]);
  }

  let users = sqlx::query_as::<_, ProfileSummary>(
    r#"
        SELECT id, username, discrim
        FROM users
        WHERE id = ANY($1)
        "#,
  )
  .bind(user_ids)
  .fetch_all(postgres.pool())
  .await?;

  Ok(users)
}
