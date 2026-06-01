use std::str::FromStr;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, prelude::Type};

#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Type,
)]
#[sqlx(type_name = "relationship_type")]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum RelationshipType {
  Friend,
  IncomingRequest,
  OutgoingRequest,
  Block,
}

impl ToString for RelationshipType {
  fn to_string(&self) -> String {
    serde_json::to_string(self)
      .unwrap()
      .trim_matches('"')
      .to_string()
  }
}

impl FromStr for RelationshipType {
  type Err = serde_json::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    serde_json::from_str(&format!("\"{s}\""))
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct Relationship {
  pub user_id: String,
  pub target_id: String,

  #[sqlx(rename = "type")]
  pub relationship_type: RelationshipType,

  pub nickname: Option<String>,

  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
