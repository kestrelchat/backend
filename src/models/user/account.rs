use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Account {
  pub id: String,
  pub email: String,
  pub email_verified: bool,
  pub password: String,
  pub birthday: NaiveDate,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub totp_secret: Option<String>,
}
