use crate::{
  adapters::crypto::hasher,
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::{
    postgres::{
      connection::Database,
      error::DatabaseError,
      operations::{
        account::get_account_by_id,
        sessions::{
          fetch_session::lookup_session,
          revoke_all_sessions as postgres_revoke_all_sessions,
          revoke_session as postgres_revoke_session,
        },
      },
    },
    redis::{
      connection::Redis,
      operations::sessions::{
        revoke_all_sessions as redis_revoke_all_sessions,
        revoke_session as redis_revoke_session,
      },
    },
  },
  errors::AppError,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct RevokeSessionRequest {
  password: String,
}

#[openapi(tag = "Sessions")]
#[post("/logout")]
pub async fn revoke_current_session(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  auth_ctx: AuthContext,
) -> Result<(), AppError> {
  let session_id = auth_ctx.session_id;

  postgres_revoke_session(postgres, &session_id)
    .await
    .map_err(AppError::from)?;

  redis_revoke_session(redis, &session_id)
    .await
    .map_err(AppError::from)?;

  Ok(())
}

#[openapi(tag = "Sessions")]
#[delete("/sessions", data = "<req>")]
pub async fn revoke_all_sessions(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  auth_ctx: AuthContext,
  req: Json<RevokeSessionRequest>,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;
  let current_token = auth_ctx.token;
  let current_session = auth_ctx.session_id;

  check_password(postgres, &user_id, &req.password).await?;

  postgres_revoke_all_sessions(postgres, &user_id, &current_session)
    .await
    .map_err(AppError::from)?;

  redis_revoke_all_sessions(redis, &user_id, &current_token)
    .await
    .map_err(AppError::from)?;

  Ok(())
}

#[openapi(tag = "Sessions")]
#[delete("/sessions/<session_id>", data = "<req>")]
pub async fn revoke_session(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  auth_ctx: AuthContext,
  session_id: &str,
  req: Json<RevokeSessionRequest>,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;

  // Check if this session belongs to user
  lookup_session(postgres, &user_id, session_id).await?;

  check_password(postgres, &user_id, &req.password).await?;

  postgres_revoke_session(postgres, session_id)
    .await
    .map_err(AppError::from)?;

  redis_revoke_session(redis, session_id)
    .await
    .map_err(AppError::from)?;

  Ok(())
}

async fn check_password(
  postgres: &Database,
  user_id: &str,
  password: &str,
) -> Result<(), AppError> {
  let account =
    get_account_by_id(postgres, user_id)
      .await
      .map_err(|e| match e {
        DatabaseError::NotFound => {
          AppError::unauthorized("INVALID_CREDENTIALS")
        }
        other => AppError::from(other),
      })?;

  hasher::password_verify(password.as_bytes(), &account.password)
    .await
    .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

  Ok(())
}
