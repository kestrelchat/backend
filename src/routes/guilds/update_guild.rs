use kestrel_postgres::{
  connection::Database,
  operations::guilds::{
    get_guild as pg_get_guild, update_guild as pg_update_guild,
  },
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  guards::rate_limit::WithinRateLimit,
  utils::{auth_context::AuthContext, errors::AppError},
};

#[derive(Deserialize, JsonSchema)]
pub struct UpdateGuildRequest {
  pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct UpdateGuildResponse {
  pub id: String,
  pub name: String,
  pub owner_id: String,
}

#[openapi(tag = "Guilds")]
#[patch("/<guild_id>", data = "<req>")]
pub async fn update_guild(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  guild_id: &str,
  req: Json<UpdateGuildRequest>,
) -> Result<Json<UpdateGuildResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  if req.name.trim().is_empty() {
    return Err(AppError::bad_request("GUILD_NAME_EMPTY"));
  }

  let guild = pg_get_guild(postgres, guild_id, &user_id)
    .await
    .map_err(|_| AppError::not_found("GUILD_NOT_FOUND"))?;

  if guild.owner_id != user_id {
    return Err(AppError::forbidden("NOT_GUILD_OWNER"));
  }

  let updated = pg_update_guild(postgres, guild_id, &req.name)
    .await
    .map_err(|e| match e {
      kestrel_postgres::error::DatabaseError::CheckViolation(ref c)
        if c == "guild_name_length" =>
      {
        AppError::bad_request("GUILD_NAME_INVALID_LENGTH")
      }
      other => AppError::from(other),
    })?;

  Ok(Json(UpdateGuildResponse {
    id: updated.id,
    name: updated.name,
    owner_id: updated.owner_id,
  }))
}
