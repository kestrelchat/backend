use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    operations::channels::get_channel as postgres_get_channel,
  },
  errors::AppError,
  models::Channel,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum GetChannelResponse {
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

impl From<Channel> for GetChannelResponse {
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
#[get("/<channel_id>")]
pub async fn get_channel(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  channel_id: &str,
) -> Result<Json<GetChannelResponse>, AppError> {
  let channel = postgres_get_channel(postgres, channel_id, &auth_ctx.user_id)
    .await
    .map_err(AppError::from)?;

  Ok(Json(channel.into()))
}
