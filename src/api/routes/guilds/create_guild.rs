use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    operations::guilds::create_guild as postgres_create_guild,
  },
  errors::AppError,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct CreateGuildRequest {
  pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct CreateGuildResponse {
  pub id: String,
  pub name: String,
  pub owner_id: String,
}

#[openapi(tag = "Guilds")]
#[post("/", data = "<req>")]
pub async fn create_guild(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  req: Json<CreateGuildRequest>,
) -> Result<Json<CreateGuildResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  if req.name.trim().is_empty() {
    return Err(AppError::bad_request("GUILD_NAME_EMPTY"));
  }

  let guild = postgres_create_guild(postgres, &req.name, &user_id)
    .await
    .map_err(|e| match e {
      crate::database::postgres::error::DatabaseError::CheckViolation(
        ref c,
      ) if c == "guild_name_length" => {
        AppError::bad_request("GUILD_NAME_INVALID_LENGTH")
      }
      other => AppError::from(other),
    })?;

  Ok(Json(CreateGuildResponse {
    id: guild.id,
    name: guild.name,
    owner_id: guild.owner_id,
  }))
}
