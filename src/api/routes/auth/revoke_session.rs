use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::{
    postgres::{
      connection::Database,
      operations::sessions::{
        revoke_all_sessions as postgres_revoke_all_sessions,
        revoke_session as postgres_revoke_session,
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
use rocket::State;
use rocket_okapi::openapi;

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
#[delete("/sessions")]
pub async fn revoke_all_sessions(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  auth_ctx: AuthContext,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;
  let current_token = auth_ctx.token;
  let current_session = auth_ctx.session_id;

  postgres_revoke_all_sessions(postgres, &user_id, &current_session)
    .await
    .map_err(AppError::from)?;

  redis_revoke_all_sessions(redis, &user_id, &current_token)
    .await
    .map_err(AppError::from)?;

  Ok(())
}

#[openapi(tag = "Sessions")]
#[delete("/sessions/<session_id>")]
pub async fn revoke_session(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  _auth_ctx: AuthContext,
  session_id: &str,
) -> Result<(), AppError> {
  postgres_revoke_session(postgres, session_id)
    .await
    .map_err(AppError::from)?;

  redis_revoke_session(redis, session_id)
    .await
    .map_err(AppError::from)?;

  Ok(())
}
