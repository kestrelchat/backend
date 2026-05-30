use kestrel_postgres::{
  connection::Database,
  operations::sessions::{
    revoke_all_sessions as postgres_revoke_all_sessions,
    revoke_session as postgres_revoke_session,
  },
};
use kestrel_redis::{
  connection::Redis,
  operations::sessions::{
    revoke_all_sessions as redis_revoke_all_sessions,
    revoke_session as redis_revoke_session,
  },
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::{auth_context::AuthContext, errors::AppError};

#[derive(Serialize, JsonSchema)]
pub struct LogoutResponse {
  pub success: bool,
}

#[openapi(tag = "Sessions")]
#[delete("/logout")]
pub async fn revoke_current_session(
  redis: &State<Redis>,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
) -> Result<Json<LogoutResponse>, AppError> {
  let session_id = auth_ctx.session_id;

  postgres_revoke_session(postgres, &session_id)
    .await
    .map_err(AppError::from)?;

  redis_revoke_session(redis, &session_id)
    .await
    .map_err(AppError::from)?;

  Ok(Json(LogoutResponse { success: true }))
}

#[openapi(tag = "Sessions")]
#[delete("/sessions")]
pub async fn revoke_all_sessions(
  redis: &State<Redis>,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
) -> Result<Json<LogoutResponse>, AppError> {
  let user_id = auth_ctx.user_id;
  let current_token = auth_ctx.token;

  postgres_revoke_all_sessions(postgres, &user_id, &current_token)
    .await
    .map_err(AppError::from)?;

  redis_revoke_all_sessions(redis, &user_id, &current_token)
    .await
    .map_err(AppError::from)?;

  Ok(Json(LogoutResponse { success: true }))
}

#[openapi(tag = "Sessions")]
#[delete("/sessions/<session_id>")]
pub async fn revoke_session(
  redis: &State<Redis>,
  postgres: &State<Database>,
  _auth_ctx: AuthContext,
  session_id: &str,
) -> Result<Json<LogoutResponse>, AppError> {
  postgres_revoke_session(postgres, session_id)
    .await
    .map_err(AppError::from)?;

  redis_revoke_session(redis, session_id)
    .await
    .map_err(AppError::from)?;

  Ok(Json(LogoutResponse { success: true }))
}
