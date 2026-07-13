use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    error::DatabaseError,
    operations::channels::update_channel::{
      UpdateChannelInput, update_channel as postgres_update_channel,
    },
  },
  errors::AppError,
  models::channel::UpdatableChannel,
};

// TODO: maybe use custom enum instead
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
  T: serde::Deserialize<'de>,
  D: serde::Deserializer<'de>,
{
  serde::Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum UpdateChannelRequest {
  Guild {
    #[serde(default, deserialize_with = "deserialize_some")]
    category_id: Option<Option<String>>,
    position: Option<i32>,
    identifier: Option<String>,
    display_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    emoji_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    topic: Option<Option<String>>,
  },
  Group {
    owner_id: Option<String>,
    display_name: Option<String>,
  },
}

impl From<UpdateChannelRequest> for UpdateChannelInput {
  fn from(req: UpdateChannelRequest) -> Self {
    match req {
      UpdateChannelRequest::Guild {
        category_id,
        position,
        identifier,
        display_name,
        emoji_id,
        topic,
      } => Self::Guild {
        category_id,
        position,
        identifier,
        display_name,
        emoji_id,
        topic,
      },
      UpdateChannelRequest::Group {
        owner_id,
        display_name,
      } => Self::Group {
        owner_id,
        display_name,
      },
    }
  }
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum UpdateChannelResponse {
  Guild {
    id: String,
    guild_id: String,
    identifier: String,
    display_name: String,
    topic: Option<String>,
    category_id: Option<String>,
    position: i32,
  },
  Group {
    id: String,
    owner_id: String,
    display_name: String,
  },
}

impl From<UpdatableChannel> for UpdateChannelResponse {
  fn from(channel: UpdatableChannel) -> Self {
    match channel {
      UpdatableChannel::Guild(gc) => Self::Guild {
        id: gc.channel_id,
        guild_id: gc.guild_id,
        identifier: gc.identifier,
        display_name: gc.display_name,
        topic: gc.topic,
        category_id: gc.category_id,
        position: gc.position,
      },
      UpdatableChannel::Group(gc) => Self::Group {
        id: gc.channel_id,
        owner_id: gc.owner_id,
        display_name: gc.display_name,
      },
    }
  }
}

#[openapi(tag = "Channels")]
#[patch("/<channel_id>", data = "<req>")]
pub async fn update_channel(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  channel_id: &str,
  req: Json<UpdateChannelRequest>,
) -> Result<Json<UpdateChannelResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  let channel = postgres_update_channel(
    postgres,
    &user_id,
    channel_id,
    req.into_inner().into(),
  )
  .await
  .map_err(|e| match e {
    DatabaseError::InvalidOperation(ref c) => match c.as_str() {
      "CHANNEL_NOT_FOUND" => AppError::not_found("CHANNEL_NOT_FOUND"),
      "NO_FIELD_TO_CHANGE" => AppError::bad_request("NO_FIELD_TO_CHANGE"),
      _ => AppError::bad_request("INVALID_OPERATION"),
    },
    DatabaseError::CheckViolation(ref c) => match c.as_str() {
      "guild_channels_identifier_format" => {
        AppError::bad_request("CHANNEL_IDENTIFIER_INVALID")
      }
      "guild_channels_identifier_length"
      | "guild_channels_display_name_length"
      | "group_channels_display_name_length" => {
        AppError::bad_request("CHANNEL_NAME_INVALID_LENGTH")
      }
      _ => AppError::bad_request("INVALID_INPUT_FORMAT"),
    },
    other => AppError::from(other),
  })?;

  Ok(Json(channel.into()))
}
