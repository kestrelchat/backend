use chrono::Utc;
use sqlx::{Postgres, Transaction};
use ulid::Ulid;

use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::{Channel, DirectChannel, GroupChannel, GuildChannel};

pub enum CreateChannelInput {
  Guild {
    guild_id: String,
    identifier: String,
    category_id: Option<String>,
  },
  Direct {
    recipient_id: String,
  },
  Group {
    display_name: String,
  },
}

pub async fn create_channel(
  postgres: &Database,
  user_id: &str,
  input: CreateChannelInput,
) -> Result<Channel, DatabaseError> {
  let mut tx = postgres.pool().begin().await?;

  let channel = match input {
    CreateChannelInput::Guild {
      guild_id,
      identifier,
      category_id,
    } => {
      create_guild_channel(&mut tx, user_id, guild_id, identifier, category_id)
        .await?
    }
    CreateChannelInput::Direct { recipient_id } => {
      create_direct_channel(&mut tx, user_id, recipient_id).await?
    }
    CreateChannelInput::Group { display_name } => {
      create_group_channel(&mut tx, user_id, display_name).await?
    }
  };

  tx.commit().await?;
  Ok(channel)
}

async fn create_base_channel(
  tx: &mut Transaction<'_, Postgres>,
  id: &str,
  r#type: &str,
) -> Result<(), DatabaseError> {
  let now = Utc::now();
  sqlx::query("INSERT INTO channels (id, type, created_at, updated_at) VALUES ($1, $2::channel_type, $3, $3)")
    .bind(id)
    .bind(r#type)
    .bind(now)
    .execute(&mut **tx)
    .await?;
  Ok(())
}

async fn create_guild_channel(
  tx: &mut Transaction<'_, Postgres>,
  user_id: &str,
  guild_id: String,
  identifier: String,
  category_id: Option<String>,
) -> Result<Channel, DatabaseError> {
  let is_member: bool = sqlx::query_scalar(
    "SELECT EXISTS (SELECT 1 FROM guild_members WHERE guild_id = $1 AND user_id = $2)",
  )
  .bind(&guild_id)
  .bind(user_id)
  .fetch_one(&mut **tx)
  .await?;

  if !is_member {
    return Err(DatabaseError::InvalidOperation("GUILD_NOT_FOUND".into()));
  }

  let position: i32 = sqlx::query_scalar(
    "SELECT COALESCE(MAX(position), -1) + 1 FROM guild_channels WHERE guild_id = $1 AND category_id IS NOT DISTINCT FROM $2::CHAR(26)",
  )
  .bind(&guild_id)
  .bind(&category_id)
  .fetch_one(&mut **tx)
  .await?;

  let id = Ulid::new().to_string();
  create_base_channel(tx, &id, "GUILD_TEXT").await?;

  let guild_channel = sqlx::query_as::<_, GuildChannel>(
    "INSERT INTO guild_channels (channel_id, guild_id, category_id, position, identifier, display_name)
     VALUES ($1, $2, $3, $4, $5, $5)
     RETURNING channel_id, guild_id, category_id, position, identifier, display_name, emoji_id, topic",
  )
  .bind(&id)
  .bind(&guild_id)
  .bind(&category_id)
  .bind(position)
  .bind(&identifier)
  .fetch_one(&mut **tx)
  .await?;

  Ok(Channel::Guild(guild_channel))
}

async fn create_direct_channel(
  tx: &mut Transaction<'_, Postgres>,
  user_id: &str,
  recipient_id: String,
) -> Result<Channel, DatabaseError> {
  if recipient_id == user_id {
    return Err(DatabaseError::InvalidOperation("CANNOT_DM_SELF".into()));
  }

  let exists: bool =
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
      .bind(&recipient_id)
      .fetch_one(&mut **tx)
      .await?;

  if !exists {
    return Err(DatabaseError::InvalidOperation(
      "RECIPIENT_NOT_FOUND".into(),
    ));
  }

  let id = Ulid::new().to_string();
  create_base_channel(tx, &id, "DIRECT").await?;

  let (user_a, user_b) = if user_id < recipient_id.as_str() {
    (user_id, recipient_id.as_str())
  } else {
    (recipient_id.as_str(), user_id)
  };

  let direct_channel = sqlx::query_as::<_, DirectChannel>(
    "INSERT INTO direct_channels (channel_id, user_a, user_b) VALUES ($1, $2, $3) RETURNING channel_id, user_a, user_b",
  )
  .bind(&id)
  .bind(user_a)
  .bind(user_b)
  .fetch_one(&mut **tx)
  .await?;

  Ok(Channel::Direct(direct_channel))
}

async fn create_group_channel(
  tx: &mut Transaction<'_, Postgres>,
  user_id: &str,
  display_name: String,
) -> Result<Channel, DatabaseError> {
  let id = Ulid::new().to_string();
  create_base_channel(tx, &id, "GROUP").await?;

  let group_channel = sqlx::query_as::<_, GroupChannel>(
    "INSERT INTO group_channels (channel_id, owner_id, display_name)
     VALUES ($1, $2, $3)
     RETURNING channel_id, owner_id, display_name",
  )
  .bind(&id)
  .bind(user_id)
  .bind(&display_name)
  .fetch_one(&mut **tx)
  .await?;

  Ok(Channel::Group(group_channel))
}
