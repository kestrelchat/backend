use chrono::Utc;
use sqlx::{PgExecutor, query_as};

use crate::error::DatabaseError;
use kestrel_common::models::User;

pub async fn create_user(
  db: impl PgExecutor<'_>,
  id: String,
  username: &str,
) -> Result<User, DatabaseError> {
  let created_at = Utc::now();
  let updated_at = created_at;

  let discrim = "0000";

  let user = query_as::<_, User>(
    r#"
        INSERT INTO users (id, username, discrim, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, discrim, created_at, updated_at
        "#,
  )
  .bind(id)
  .bind(username)
  .bind(discrim)
  .bind(created_at)
  .bind(updated_at)
  .fetch_one(db)
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(user)
}
