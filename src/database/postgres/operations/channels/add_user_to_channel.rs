use chrono::Utc;
use sqlx::{Postgres, Transaction};

use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::GroupMember;

pub async fn add_user_to_channel(
  postgres: &Database,
  requester_id: &str,
  channel_id: &str,
  user_id: &str,
  group_member_limit: i64,
) -> Result<GroupMember, DatabaseError> {
  let mut tx = postgres.pool().begin().await?;

  let channel_type: String =
    sqlx::query_scalar("SELECT type::text FROM channels WHERE id = $1")
      .bind(channel_id)
      .fetch_optional(&mut *tx)
      .await
      .map_err(DatabaseError::from_sqlx)?
      .ok_or(DatabaseError::InvalidOperation("CHANNEL_NOT_FOUND".into()))?;

  let member = match channel_type.as_str() {
    "GROUP" => {
      add_group_member(&mut tx, requester_id, channel_id, user_id, group_member_limit)
        .await?
    }
    "DIRECT" => {
      return Err(DatabaseError::InvalidOperation(
        "CANNOT_ADD_USER_TO_DIRECT".into(),
      ));
    }
    "GUILD_TEXT" => {
      return Err(DatabaseError::InvalidOperation(
        "CANNOT_ADD_USER_TO_GUILD_CHANNEL".into(),
      ));
    }
    _ => return Err(DatabaseError::NotFound),
  };

  tx.commit().await?;
  Ok(member)
}

async fn add_group_member(
  tx: &mut Transaction<'_, Postgres>,
  requester_id: &str,
  channel_id: &str,
  user_id: &str,
  group_member_limit: i64,
) -> Result<GroupMember, DatabaseError> {
  let is_owner: bool = sqlx::query_scalar(
    "SELECT EXISTS (SELECT 1 FROM group_channels WHERE channel_id = $1 AND owner_id = $2)",
  )
  .bind(channel_id)
  .bind(requester_id)
  .fetch_one(&mut **tx)
  .await?;

  if !is_owner {
    return Err(DatabaseError::InvalidOperation("CHANNEL_NOT_FOUND".into()));
  }

  if requester_id == user_id {
    return Err(DatabaseError::InvalidOperation("CANNOT_ADD_SELF".into()));
  }

  let is_member: bool = sqlx::query_scalar(
    "SELECT EXISTS (SELECT 1 FROM group_members WHERE channel_id = $1 AND user_id = $2)",
  )
  .bind(channel_id)
  .bind(user_id)
  .fetch_one(&mut **tx)
  .await?;

  if is_member {
    return Err(DatabaseError::InvalidOperation("ALREADY_MEMBER".into()));
  }

  let user_exists: bool =
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
      .bind(user_id)
      .fetch_one(&mut **tx)
      .await?;

  if !user_exists {
    return Err(DatabaseError::InvalidOperation("USER_NOT_FOUND".into()));
  }

  let member_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM group_members WHERE channel_id = $1")
      .bind(channel_id)
      .fetch_one(&mut **tx)
      .await?;

  if member_count >= group_member_limit {
    return Err(DatabaseError::InvalidOperation(
      "GROUP_MEMBER_LIMIT_REACHED".into(),
    ));
  }

  let member = sqlx::query_as::<_, GroupMember>(
    "INSERT INTO group_members (channel_id, user_id, joined_at)
     VALUES ($1, $2, $3)
     RETURNING channel_id, user_id, joined_at",
  )
  .bind(channel_id)
  .bind(user_id)
  .bind(Utc::now())
  .fetch_one(&mut **tx)
  .await?;

  Ok(member)
}
