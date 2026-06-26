use sqlx::query;

use crate::database::postgres::{connection::Database, error::DatabaseError};

pub async fn delete_guild(
  postgres: &Database,
  guild_id: &str,
) -> Result<(), DatabaseError> {
  let rows = query(
    r#"
        DELETE FROM guilds
        WHERE id = $1
        "#,
  )
  .bind(guild_id)
  .execute(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?
  .rows_affected();

  if rows == 0 {
    return Err(DatabaseError::NotFound);
  }

  Ok(())
}
