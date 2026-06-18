use sqlx::query_as;

use crate::{connection::Database, error::DatabaseError};
use kestrel_common::models::Guild;

pub async fn get_guild(
  db: &Database,
  guild_id: &str,
  user_id: &str,
) -> Result<Guild, DatabaseError> {
  let guild = query_as::<_, Guild>(
    r#"
        SELECT g.id, g.name, g.owner_id, g.created_at, g.updated_at
        FROM guilds g
        INNER JOIN guild_members gm ON gm.guild_id = g.id
        WHERE g.id = $1
          AND gm.user_id = $2
        "#,
  )
  .bind(guild_id)
  .bind(user_id)
  .fetch_optional(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  guild.ok_or(DatabaseError::NotFound)
}
