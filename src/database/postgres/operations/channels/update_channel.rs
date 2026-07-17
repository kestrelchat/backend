use chrono::Utc;
use sqlx::{Postgres, QueryBuilder, Transaction};

use crate::{
  database::postgres::{connection::Database, error::DatabaseError},
  models::{GroupChannel, GuildChannel, channel::UpdatableChannel},
};

pub enum UpdateChannelInput {
  Guild {
    category_id: Option<Option<String>>,
    position: Option<i32>,
    identifier: Option<String>,
    display_name: Option<String>,
    emoji_id: Option<Option<String>>,
    topic: Option<Option<String>>,
  },
  Group {
    owner_id: Option<String>,
    display_name: Option<String>,
  },
}

pub async fn update_channel(
  postgres: &Database,
  user_id: &str,
  channel_id: &str,
  input: UpdateChannelInput,
) -> Result<UpdatableChannel, DatabaseError> {
  let mut tx = postgres.pool().begin().await?;

  update_base_channel(&mut tx, channel_id).await?;

  let channel = match input {
    UpdateChannelInput::Guild {
      category_id,
      position,
      identifier,
      display_name,
      emoji_id,
      topic,
    } => {
      update_guild_channel(
        &mut tx,
        channel_id,
        category_id,
        position,
        identifier,
        display_name,
        emoji_id,
        topic,
      )
      .await?
    }
    UpdateChannelInput::Group {
      owner_id,
      display_name,
    } => {
      update_group_channel(&mut tx, user_id, channel_id, owner_id, display_name)
        .await?
    }
  };

  tx.commit().await?;
  Ok(channel)
}

async fn update_base_channel(
  tx: &mut Transaction<'_, Postgres>,
  channel_id: &str,
) -> Result<(), DatabaseError> {
  let now = Utc::now();

  let result = sqlx::query("UPDATE channels SET updated_at = $1 WHERE id = $2")
    .bind(now)
    .bind(channel_id)
    .execute(&mut **tx)
    .await?;

  if result.rows_affected() == 0 {
    return Err(DatabaseError::NotFound);
  }

  Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update_guild_channel(
  tx: &mut Transaction<'_, Postgres>,
  channel_id: &str,
  category_id: Option<Option<String>>,
  // TODO: add position updating
  _position: Option<i32>,
  identifier: Option<String>,
  display_name: Option<String>,
  emoji_id: Option<Option<String>>,
  topic: Option<Option<String>>,
) -> Result<UpdatableChannel, DatabaseError> {
  let mut query = QueryBuilder::new("UPDATE guild_channels SET ");
  let mut modified = false;

  let mut separated = query.separated(", ");

  if let Some(identifier) = identifier {
    separated
      .push("identifier = ")
      .push_bind_unseparated(identifier);
    modified = true;
  }
  if let Some(display_name) = display_name {
    separated
      .push("display_name = ")
      .push_bind_unseparated(display_name);
    modified = true;
  }
  if let Some(topic) = topic {
    separated.push("topic = ").push_bind_unseparated(topic);
    modified = true;
  }
  if let Some(category_id) = category_id {
    separated
      .push("category_id = ")
      .push_bind_unseparated(category_id);
    modified = true;
  }
  if let Some(emoji_id) = emoji_id {
    separated
      .push("emoji_id = ")
      .push_bind_unseparated(emoji_id);
    modified = true;
  }
  if !modified {
    return Err(DatabaseError::InvalidOperation("NO_FIELD_TO_CHANGE".into()));
  }
  query.push(" WHERE channel_id = ").push_bind(channel_id);
  query.push(" RETURNING *");
  let guild_channel = query
    .build_query_as::<GuildChannel>()
    .fetch_one(&mut **tx)
    .await?;

  Ok(UpdatableChannel::Guild(guild_channel))
}

async fn update_group_channel(
  tx: &mut Transaction<'_, Postgres>,
  user_id: &str,
  channel_id: &str,
  owner_id: Option<String>,
  display_name: Option<String>,
) -> Result<UpdatableChannel, DatabaseError> {
  if owner_id.is_none() && display_name.is_none() {
    return Err(DatabaseError::InvalidOperation("NO_FIELD_TO_CHANGE".into()));
  }

  let is_owner: bool = sqlx::query_scalar(
      "SELECT EXISTS (SELECT 1 FROM group_channels WHERE channel_id = $1 AND owner_id = $2)",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await?;

  if !is_owner {
    return Err(DatabaseError::InvalidOperation("CHANNEL_NOT_FOUND".into()));
  }

  let group_channel = sqlx::query_as::<_, GroupChannel>(
    "UPDATE group_channels
    SET
        owner_id = COALESCE($2, owner_id),
        display_name = COALESCE($3, display_name)
    WHERE channel_id = $1
    RETURNING *",
  )
  .bind(channel_id)
  .bind(owner_id)
  .bind(display_name)
  .fetch_one(&mut **tx)
  .await?;

  Ok(UpdatableChannel::Group(group_channel))
}
