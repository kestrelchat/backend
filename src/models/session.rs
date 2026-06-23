use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct Session {
  pub id: String,
  pub user_id: String,

  pub refresh_token: String,

  pub ip_address: Option<String>,
  pub country: Option<String>,
  pub region: Option<String>,
  pub city: Option<String>,

  pub user_agent: Option<String>,
  pub operating_system: Option<String>,
  pub platform: Option<String>,

  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub expires_at: DateTime<Utc>,

  pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSession {
  pub session_id: String,
  pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingMfaScope {
  Setup,
  Login,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingMfaKind {
  Totp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMfa {
  pub scope: PendingMfaScope,
  pub kind: PendingMfaKind,
  pub account_id: String,
  pub protected_payload: String,
}
