use std::{
  fmt::{Display, Formatter},
  str::FromStr,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipAction {
  Friend,
  Block,
}

impl Display for RelationshipType {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
    let s = match self {
      RelationshipType::Friend => "friend",
      RelationshipType::IncomingRequest => "incoming_request",
      RelationshipType::OutgoingRequest => "outgoing_request",
      RelationshipType::Block => "block",
    };

    write!(f, "{}", s)
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
