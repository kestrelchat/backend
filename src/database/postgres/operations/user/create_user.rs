use chrono::Utc;
use sqlx::{PgExecutor, query_as};

use crate::database::postgres::error::DatabaseError;
use crate::models::Profile;

pub async fn create_user(
  postgres: impl PgExecutor<'_>,
  id: String,
  username: &str,
) -> Result<Profile, DatabaseError> {
  let created_at = Utc::now();
  let updated_at = created_at;

  let discrim = "0000";

  let user = query_as::<_, Profile>(
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
  .fetch_one(postgres)
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(user)
}
