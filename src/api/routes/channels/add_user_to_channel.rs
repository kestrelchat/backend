use chrono::{DateTime, Utc};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  config::Config,
  database::postgres::{
    connection::Database,
    error::DatabaseError,
    operations::channels::add_user_to_channel as postgres_add_user_to_channel,
  },
  errors::AppError,
  models::GroupMember,
};

#[derive(Deserialize, JsonSchema)]
pub struct AddUserToChannelRequest {
  pub user_id: String,
}

#[derive(Serialize, JsonSchema)]
pub struct AddUserToChannelResponse {
  pub channel_id: String,
  pub user_id: String,
  pub joined_at: DateTime<Utc>,
}

impl From<GroupMember> for AddUserToChannelResponse {
  fn from(member: GroupMember) -> Self {
    Self {
      channel_id: member.channel_id,
      user_id: member.user_id,
      joined_at: member.joined_at,
    }
  }
}

#[openapi(tag = "Channels")]
#[post("/<channel_id>/members", data = "<req>")]
pub async fn add_user_to_channel(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  config: &State<Config>,
  auth_ctx: AuthContext,
  channel_id: &str,
  req: Json<AddUserToChannelRequest>,
) -> Result<Json<AddUserToChannelResponse>, AppError> {
  let member = postgres_add_user_to_channel(
    postgres,
    &auth_ctx.user_id,
    channel_id,
    &req.user_id,
    config.channels.group_member_limit,
  )
  .await
  .map_err(|e| match e {
    DatabaseError::InvalidOperation(ref c) => match c.as_str() {
      "CHANNEL_NOT_FOUND" => AppError::not_found("CHANNEL_NOT_FOUND"),
      "USER_NOT_FOUND" => AppError::not_found("USER_NOT_FOUND"),
      "CANNOT_ADD_SELF" => AppError::bad_request("CANNOT_ADD_SELF"),
      "ALREADY_MEMBER" => AppError::conflict("ALREADY_MEMBER"),
      "GROUP_MEMBER_LIMIT_REACHED" => {
        AppError::bad_request("GROUP_MEMBER_LIMIT_REACHED")
      }
      "CANNOT_ADD_USER_TO_DIRECT" => {
        AppError::bad_request("CANNOT_ADD_USER_TO_DIRECT")
      }
      "CANNOT_ADD_USER_TO_GUILD_CHANNEL" => {
        AppError::bad_request("CANNOT_ADD_USER_TO_GUILD_CHANNEL")
      }
      _ => AppError::bad_request("INVALID_OPERATION"),
    },
    other => AppError::from(other),
  })?;

  Ok(Json(member.into()))
}
