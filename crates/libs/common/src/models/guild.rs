use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Guild {
  pub id: String,
  pub name: String,
  pub owner_id: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct GuildMember {
  pub guild_id: String,
  pub user_id: String,
  pub joined_at: DateTime<Utc>,
}
