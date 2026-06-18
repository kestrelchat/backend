use chrono::Utc;
use sqlx::query_as;

use crate::{connection::Database, error::DatabaseError};
use kestrel_common::models::Guild;

pub async fn update_guild(
  db: &Database,
  guild_id: &str,
  name: &str,
) -> Result<Guild, DatabaseError> {
  let updated_at = Utc::now();

  let guild = query_as::<_, Guild>(
    r#"
        UPDATE guilds
        SET name = $2, updated_at = $3
        WHERE id = $1
        RETURNING id, name, owner_id, created_at, updated_at
        "#,
  )
  .bind(guild_id)
  .bind(name)
  .bind(updated_at)
  .fetch_optional(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  guild.ok_or(DatabaseError::NotFound)
}
