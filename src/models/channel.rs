use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Channel {
  pub id: String,
  #[sqlx(rename = "type")]
  pub channel_type: ChannelType,
  pub guild_id: Option<String>,
  pub category_id: Option<String>,
  pub position: i32,
  pub identifier: String,
  pub display_name: String,
  pub emoji_id: Option<String>,
  pub topic: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "channel_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelType {
  GuildText,
  Direct,
  Group,
}
