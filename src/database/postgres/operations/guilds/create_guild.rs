use chrono::Utc;
use sqlx::query_as;
use ulid::Ulid;

use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::Guild;

pub async fn create_guild(
  postgres: &Database,
  name: &str,
  owner_id: &str,
) -> Result<Guild, DatabaseError> {
  let id = Ulid::new().to_string();
  let now = Utc::now();

  let guild = query_as::<_, Guild>(
    r#"
        INSERT INTO guilds (id, name, owner_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, owner_id, created_at, updated_at
        "#,
  )
  .bind(&id)
  .bind(name)
  .bind(owner_id)
  .bind(now)
  .bind(now)
  .fetch_one(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  sqlx::query(
    r#"
        INSERT INTO guild_members (guild_id, user_id, joined_at)
        VALUES ($1, $2, $3)
        "#,
  )
  .bind(&guild.id)
  .bind(owner_id)
  .bind(now)
  .execute(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(guild)
}
