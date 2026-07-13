use chrono::{DateTime, Utc};
use sqlx::prelude::{FromRow, Type};

#[derive(Debug, Clone, PartialEq, Type)]
pub enum ChannelType {
  GuildText,
  Direct,
  Group,
}

#[derive(Debug, Clone, FromRow)]
pub struct BaseChannel {
  pub id: String,
  pub channel_type: ChannelType,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct GuildChannel {
  pub channel_id: String,
  pub guild_id: String,
  pub category_id: Option<String>,
  pub position: i32,
  pub identifier: String,
  pub display_name: String,
  pub emoji_id: Option<String>,
  pub topic: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DirectChannel {
  pub channel_id: String,
  pub user_a: String,
  pub user_b: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupChannel {
  pub channel_id: String,
  pub owner_id: String,
  pub display_name: String,
}

#[derive(Debug, Clone)]
pub enum Channel {
  Guild(GuildChannel),
  Direct(DirectChannel),
  Group(GroupChannel),
}

#[derive(Debug, Clone)]
pub enum UpdatableChannel {
  Guild(GuildChannel),
  Group(GroupChannel),
}
