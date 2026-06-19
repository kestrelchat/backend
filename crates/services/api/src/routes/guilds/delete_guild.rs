use kestrel_common::utils::hasher;
use kestrel_postgres::{
  connection::Database,
  error::DatabaseError,
  operations::{
    account::get_account_by_id,
    guilds::{delete_guild as pg_delete_guild, get_guild as pg_get_guild},
  },
};
use rocket::{State, http::Status, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::utils::{auth_context::AuthContext, errors::AppError};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, JsonSchema)]
pub struct DeleteGuildRequest {
  pub password: String,
}

#[openapi(tag = "Guilds")]
#[delete("/<guild_id>", data = "<req>")]
pub async fn delete_guild(
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  guild_id: &str,
  req: Json<DeleteGuildRequest>,
) -> Result<Status, AppError> {
  let user_id = auth_ctx.user_id;

  let guild = pg_get_guild(postgres, guild_id, &user_id)
    .await
    .map_err(|_| AppError::not_found("GUILD_NOT_FOUND"))?;

  if guild.owner_id != user_id {
    return Err(AppError::forbidden("NOT_GUILD_OWNER"));
  }

  let account =
    get_account_by_id(postgres, &user_id)
      .await
      .map_err(|e| match e {
        DatabaseError::NotFound => AppError::not_found("ACCOUNT_NOT_FOUND"),
        other => AppError::from(other),
      })?;

  // later ill implement 2fa instead if the user has 2fa enabled
  hasher::password_verify(req.password.as_bytes(), &account.password)
    .await
    .map_err(|_| AppError::unauthorized("INVALID_PASSWORD"))?;

  pg_delete_guild(postgres, guild_id)
    .await
    .map_err(AppError::from)?;

  Ok(Status::NoContent)
}
