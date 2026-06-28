use chrono::Utc;
use sqlx::query_as;
use ulid::Ulid;

use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::Channel;
use crate::models::channel::ChannelType;

#[allow(clippy::too_many_arguments)]
pub async fn create_channel(
  postgres: &Database,
  user_id: &str,
  guild_id: Option<&str>,
  recipient_id: Option<&str>,
  identifier: Option<&str>,
  category_id: Option<&str>,
) -> Result<Channel, DatabaseError> {
  match (guild_id, recipient_id) {
    (Some(guild_id), None) => {
      let identifier = identifier
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
          DatabaseError::InvalidOperation("CHANNEL_IDENTIFIER_REQUIRED".into())
        })?;

      if !identifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
      {
        return Err(DatabaseError::InvalidOperation(
          "CHANNEL_IDENTIFIER_INVALID".into(),
        ));
      }

      let is_member: bool = sqlx::query_scalar(
        r#"SELECT EXISTS (SELECT 1 FROM guild_members WHERE guild_id = $1 AND user_id = $2)"#,
      )
      .bind(guild_id)
      .bind(user_id)
      .fetch_one(postgres.pool())
      .await?;

      if !is_member {
        return Err(DatabaseError::InvalidOperation("GUILD_NOT_FOUND".into()));
      }

      let position: i32 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(position), -1) + 1
        FROM channels
        WHERE guild_id = $1
          AND category_id IS NOT DISTINCT FROM $2
        "#,
      )
      .bind(guild_id)
      .bind(category_id)
      .fetch_one(postgres.pool())
      .await?;

      let id = Ulid::new().to_string();
      let now = Utc::now();

      let channel = query_as::<_, Channel>(
        r#"
        INSERT INTO channels (id, type, guild_id, category_id, position, identifier, display_name, topic, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, type, guild_id, category_id, position, identifier, display_name, emoji_id, topic, created_at, updated_at
        "#,
      )
      .bind(&id)
      .bind(&ChannelType::GuildText)
      .bind(guild_id)
      .bind(category_id)
      .bind(position)
      .bind(&identifier)
      .bind(&identifier) //display name will be the identifier by default.
      .bind(None::<&str>)
      .bind(now)
      .bind(now)
      .fetch_one(postgres.pool())
      .await?;

      Ok(channel)
    }

    (None, Some(recipient_id)) => {
      if recipient_id == user_id {
        return Err(DatabaseError::InvalidOperation("CANNOT_DM_SELF".into()));
      }

      let exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)"#,
      )
      .bind(recipient_id)
      .fetch_one(postgres.pool())
      .await?;

      if !exists {
        return Err(DatabaseError::InvalidOperation(
          "RECIPIENT_NOT_FOUND".into(),
        ));
      }

      let id = Ulid::new().to_string();
      let now = Utc::now();

      let display_name: String =
        sqlx::query_scalar(r#"SELECT username FROM users WHERE id = $1"#)
          .bind(recipient_id)
          .fetch_one(postgres.pool())
          .await?;

      let channel = query_as::<_, Channel>(
        r#"
        INSERT INTO channels (id, type, guild_id, category_id, position, identifier, display_name, topic, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, type, guild_id, category_id, position, identifier, display_name, emoji_id, topic, created_at, updated_at
        "#,
      )
      .bind(&id)
      .bind(&ChannelType::Direct)
      .bind(None::<&str>)
      .bind(None::<&str>)
      .bind(0_i32)
      .bind(&id)
      .bind(&display_name)
      .bind(None::<&str>)
      .bind(now)
      .bind(now)
      .fetch_one(postgres.pool())
      .await?;

      Ok(channel)
    }

    (None, None) => Err(DatabaseError::InvalidOperation(
      "GUILD_ID_OR_RECIPIENT_REQUIRED".into(),
    )),

    (Some(_), Some(_)) => Err(DatabaseError::InvalidOperation(
      "BOTH_GUILD_ID_AND_RECIPIENT".into(),
    )),
  }
}
