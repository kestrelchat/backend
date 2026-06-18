use sqlx::query;

use crate::{connection::Database, error::DatabaseError};

pub async fn delete_guild(
  db: &Database,
  guild_id: &str,
) -> Result<(), DatabaseError> {
  let rows = query(
    r#"
        DELETE FROM guilds
        WHERE id = $1
        "#,
  )
  .bind(guild_id)
  .execute(db.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?
  .rows_affected();

  if rows == 0 {
    return Err(DatabaseError::NotFound);
  }

  Ok(())
}
