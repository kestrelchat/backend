use sqlx::query_as;

use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::{Channel, DirectChannel, GroupChannel, GuildChannel};

pub async fn get_channel(
  postgres: &Database,
  channel_id: &str,
  user_id: &str,
) -> Result<Channel, DatabaseError> {
  let channel_type: String =
    sqlx::query_scalar("SELECT type::text FROM channels WHERE id = $1")
      .bind(channel_id)
      .fetch_optional(postgres.pool())
      .await
      .map_err(DatabaseError::from_sqlx)?
      .ok_or(DatabaseError::NotFound)?;

  match channel_type.as_str() {
    "GUILD_TEXT" => {
      let guild_channel = query_as::<_, GuildChannel>(
        r#"
        SELECT gc.channel_id, gc.guild_id, gc.category_id, gc.position,
               gc.identifier, gc.display_name, gc.emoji_id, gc.topic
        FROM guild_channels gc
        INNER JOIN guild_members gm ON gm.guild_id = gc.guild_id AND gm.user_id = $2
        WHERE gc.channel_id = $1
        "#,
      )
      .bind(channel_id)
      .bind(user_id)
      .fetch_optional(postgres.pool())
      .await
      .map_err(DatabaseError::from_sqlx)?
      .ok_or(DatabaseError::NotFound)?;

      Ok(Channel::Guild(guild_channel))
    }
    "DIRECT" => {
      let direct_channel = query_as::<_, DirectChannel>(
        r#"
        SELECT channel_id, user_a, user_b
        FROM direct_channels
        WHERE channel_id = $1
          AND ($2 = user_a OR $2 = user_b)
        "#,
      )
      .bind(channel_id)
      .bind(user_id)
      .fetch_optional(postgres.pool())
      .await
      .map_err(DatabaseError::from_sqlx)?
      .ok_or(DatabaseError::NotFound)?;

      Ok(Channel::Direct(direct_channel))
    }
    "GROUP" => {
      let group_channel = query_as::<_, GroupChannel>(
        r#"
        SELECT gc.channel_id, gc.owner_id, gc.display_name
        FROM group_channels gc
        INNER JOIN group_members gm
            ON gm.channel_id = gc.channel_id
        WHERE gc.channel_id = $1
          AND gm.user_id = $2
        "#,
      )
      .bind(channel_id)
      .bind(user_id)
      .fetch_optional(postgres.pool())
      .await
      .map_err(DatabaseError::from_sqlx)?
      .ok_or(DatabaseError::NotFound)?;

      Ok(Channel::Group(group_channel))
    }
    _ => Err(DatabaseError::NotFound),
  }
}
