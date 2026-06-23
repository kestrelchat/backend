use chrono::{DateTime, Utc};
use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct Profile {
  pub id: String,
  pub username: String,
  pub discrim: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ProfileSummary {
  pub id: String,
  pub username: String,
  pub discrim: String,
}
