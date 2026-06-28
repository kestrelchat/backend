use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database, error::DatabaseError,
    operations::channels::create_channel as postgres_create_channel,
  },
  errors::AppError,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct CreateChannelRequest {
  pub guild_id: Option<String>,
  pub recipient_id: Option<String>,
  pub identifier: Option<String>,
  pub category_id: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct CreateChannelResponse {
  pub id: String,
  #[schemars(with = "String")]
  pub channel_type: String,
  pub guild_id: Option<String>,
  pub identifier: String,
  pub display_name: String,
  pub topic: Option<String>,
  pub category_id: Option<String>,
  pub position: i32,
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

  let channel = postgres_create_channel(
    postgres,
    &user_id,
    req.guild_id.as_deref(),
    req.recipient_id.as_deref(),
    req.identifier.as_deref(),
    req.category_id.as_deref(),
  )
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
    DatabaseError::UniqueViolation(_) => {
      AppError::conflict("CHANNEL_IDENTIFIER_ALREADY_EXISTS")
    }
    DatabaseError::CheckViolation(ref c)
      if c == "channel_identifier_format" =>
    {
      AppError::bad_request("CHANNEL_IDENTIFIER_INVALID")
    }
    DatabaseError::CheckViolation(ref c)
      if c == "channel_identifier_length"
        || c == "channel_display_name_length" =>
    {
      AppError::bad_request("CHANNEL_NAME_INVALID_LENGTH")
    }
    DatabaseError::ForeignKeyViolation => {
      AppError::bad_request("INVALID_REFERENCE")
    }
    other => AppError::from(other),
  })?;

  Ok(Json(CreateChannelResponse {
    id: channel.id,
    channel_type: format!("{:?}", channel.channel_type),
    guild_id: channel.guild_id,
    identifier: channel.identifier,
    display_name: channel.display_name,
    topic: channel.topic,
    category_id: channel.category_id,
    position: channel.position,
  }))
}
