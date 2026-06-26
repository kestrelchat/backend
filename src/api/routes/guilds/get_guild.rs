use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database, operations::guilds::get_guild as postgres_get_guild,
  },
  errors::AppError,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct GetGuildResponse {
  pub id: String,
  pub name: String,
  pub owner_id: String,
}

#[openapi(tag = "Guilds")]
#[get("/<guild_id>")]
pub async fn get_guild(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  guild_id: &str,
) -> Result<Json<GetGuildResponse>, AppError> {
  let guild = postgres_get_guild(postgres, guild_id, &auth_ctx.user_id)
    .await
    .map_err(AppError::from)?;

  Ok(Json(GetGuildResponse {
    id: guild.id,
    name: guild.name,
    owner_id: guild.owner_id,
  }))
}
