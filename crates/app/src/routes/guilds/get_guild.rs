use kestrel_postgres::{
  connection::Database, operations::guilds::get_guild as pg_get_guild,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{auth_context::AuthContext, errors::AppError};

#[derive(Serialize, JsonSchema)]
pub struct GetGuildResponse {
  pub id: String,
  pub name: String,
  pub owner_id: String,
}

#[openapi(tag = "Guilds")]
#[get("/<guild_id>")]
pub async fn get_guild(
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  guild_id: &str,
) -> Result<Json<GetGuildResponse>, AppError> {
  let guild = pg_get_guild(postgres, guild_id, &auth_ctx.user_id)
    .await
    .map_err(AppError::from)?;

  Ok(Json(GetGuildResponse {
    id: guild.id,
    name: guild.name,
    owner_id: guild.owner_id,
  }))
}
