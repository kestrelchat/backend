use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    error::DatabaseError,
    operations::channels::create_channel::{
      CreateChannelInput, create_channel as postgres_create_channel,
    },
  },
  errors::AppError,
  models::Channel,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum CreateChannelRequest {
  Guild {
    guild_id: String,
    identifier: String,
    category_id: Option<String>,
  },
  Direct {
    recipient_id: String,
  },
  Group {
    display_name: String,
  },
}

impl From<CreateChannelRequest> for CreateChannelInput {
  fn from(req: CreateChannelRequest) -> Self {
    match req {
      CreateChannelRequest::Guild {
        guild_id,
        identifier,
        category_id,
      } => Self::Guild {
        guild_id,
        identifier,
        category_id,
      },
      CreateChannelRequest::Direct { recipient_id } => {
        Self::Direct { recipient_id }
      }
      CreateChannelRequest::Group { display_name } => {
        Self::Group { display_name }
      }
    }
  }
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum CreateChannelResponse {
  Guild {
    id: String,
    guild_id: String,
    identifier: String,
    display_name: String,
    topic: Option<String>,
    category_id: Option<String>,
    position: i32,
  },
  Direct {
    id: String,
    user_a: String,
    user_b: String,
  },
  Group {
    id: String,
    owner_id: String,
    display_name: String,
  },
}

impl From<Channel> for CreateChannelResponse {
  fn from(channel: Channel) -> Self {
    match channel {
      Channel::Guild(gc) => Self::Guild {
        id: gc.channel_id,
        guild_id: gc.guild_id,
        identifier: gc.identifier,
        display_name: gc.display_name,
        topic: gc.topic,
        category_id: gc.category_id,
        position: gc.position,
      },
      Channel::Direct(dc) => Self::Direct {
        id: dc.channel_id,
        user_a: dc.user_a,
        user_b: dc.user_b,
      },
      Channel::Group(gc) => Self::Group {
        id: gc.channel_id,
        owner_id: gc.owner_id,
        display_name: gc.display_name,
      },
    }
  }
}

#[openapi(tag = "Channels")]
#[post("/", data = "<req>")]
pub async fn create_channel(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  req: Json<CreateChannelRequest>,
) -> Result<Json<CreateChannelResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  let channel =
    postgres_create_channel(postgres, &user_id, req.into_inner().into())
      .await
      .map_err(|e| match e {
        DatabaseError::InvalidOperation(ref c) => match c.as_str() {
          "GUILD_ID_OR_RECIPIENT_REQUIRED" => {
            AppError::bad_request("GUILD_ID_OR_RECIPIENT_REQUIRED")
          }
          "BOTH_GUILD_ID_AND_RECIPIENT" => {
            AppError::bad_request("BOTH_GUILD_ID_AND_RECIPIENT")
          }
          "CHANNEL_IDENTIFIER_REQUIRED" => {
            AppError::bad_request("CHANNEL_IDENTIFIER_REQUIRED")
          }
          "CHANNEL_IDENTIFIER_INVALID" => {
            AppError::bad_request("CHANNEL_IDENTIFIER_INVALID")
          }
          "CANNOT_DM_SELF" => AppError::bad_request("CANNOT_DM_SELF"),
          "RECIPIENT_NOT_FOUND" => AppError::not_found("RECIPIENT_NOT_FOUND"),
          "GUILD_NOT_FOUND" => AppError::not_found("GUILD_NOT_FOUND"),
          _ => AppError::bad_request("INVALID_OPERATION"),
        },
        DatabaseError::UniqueViolation(ref c) => match c.as_str() {
          "direct_channels_unique_pair" => {
            AppError::conflict("DIRECT_CHANNEL_ALREADY_EXISTS")
          }
          _ => AppError::conflict("CHANNEL_IDENTIFIER_ALREADY_EXISTS"),
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
          "direct_channels_user_order" => {
            AppError::bad_request("INVALID_USER_ORDER")
          }
          _ => AppError::bad_request("INVALID_INPUT_FORMAT"),
        },
        DatabaseError::ForeignKeyViolation => {
          AppError::bad_request("INVALID_REFERENCE")
        }
        other => AppError::from(other),
      })?;

  Ok(Json(channel.into()))
}
