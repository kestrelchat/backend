use kestrel_common::models::user::UserSummary;

use crate::{connection::Database, error::DatabaseError};

pub async fn batch_get_user_summaries(
  db: &Database,
  user_ids: &[String],
) -> Result<Vec<UserSummary>, DatabaseError> {
  if user_ids.is_empty() {
    return Ok(vec![]);
  }

  let users = sqlx::query_as::<_, UserSummary>(
    r#"
        SELECT id, username, discrim
        FROM users
        WHERE id = ANY($1)
        "#,
  )
  .bind(user_ids)
  .fetch_all(db.pool())
  .await?;

  Ok(users)
}
