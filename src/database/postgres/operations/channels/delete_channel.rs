use crate::database::postgres::{connection::Database, error::DatabaseError};

pub async fn delete_channel(
  postgres: &Database,
  user_id: &str,
  channel_id: &str,
) -> Result<(), DatabaseError> {
  // TODO: add proper authorization checks for guild channel deletion
  let result = sqlx::query(
    r#"
        DELETE FROM channels AS c
        WHERE c.id = $1
        AND (
            (
                c.type = 'GROUP'
                AND EXISTS (
                    SELECT 1
                    FROM group_channels AS gc
                    WHERE gc.channel_id = c.id
                    AND gc.owner_id = $2
                )
            )
            OR
            (
                c.type = 'DIRECT'
                AND EXISTS (
                    SELECT 1
                    FROM direct_channels AS dc
                    WHERE dc.channel_id = c.id
                    AND (
                        dc.user_a = $2
                        OR dc.user_b = $2
                    )
                )
            )
            OR
            (
                c.type = 'GUILD_TEXT'
                AND EXISTS (
                    SELECT 1
                    FROM guild_channels AS gch
                    JOIN guild_members AS gm
                        ON gm.guild_id = gch.guild_id
                    WHERE gch.channel_id = c.id
                    AND gm.user_id = $2
                )
            )
        )
    "#,
  )
  .bind(channel_id)
  .bind(user_id)
  .execute(postgres.pool())
  .await?;

  if result.rows_affected() == 0 {
    return Err(DatabaseError::NotFound);
  }

  Ok(())
}
