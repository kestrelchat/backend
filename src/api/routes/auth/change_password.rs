use crate::{
  adapters::crypto::hasher,
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::{
    postgres::{
      connection::Database,
      error::DatabaseError,
      operations::{
        account::{
          change_password as postgres_change_password, get_account_by_id,
        },
        sessions::revoke_all_sessions as postgres_revoke_all_sessions,
      },
    },
    redis::{
      connection::Redis,
      operations::sessions::revoke_all_sessions as redis_revoke_all_sessions,
    },
  },
  errors::AppError,
  models::{ValidationError, user::password::Password},
};

use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct ChangePasswordRequest {
  old_password: String,
  new_password: String,
}

#[openapi(tag = "Authentication")]
#[post("/password/change", data = "<req>")]
pub async fn change_password(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  auth_ctx: AuthContext,
  req: Json<ChangePasswordRequest>,
) -> Result<(), AppError> {
  let account = match get_account_by_id(postgres, &auth_ctx.user_id).await {
    Ok(acc) => acc,

    Err(e) => match e {
      DatabaseError::NotFound => {
        return Err(AppError::unauthorized("INVALID_CREDENTIALS"));
      }

      other => return Err(AppError::from(other)),
    },
  };

  hasher::password_verify(req.old_password.as_bytes(), &account.password)
    .await
    .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

  let password =
    Password::new(&req.new_password).map_err(ValidationError::Password)?;

  let hashed_password = hasher::password_hash(Zeroizing::new(
    password.as_ref().as_bytes().to_vec(),
  ))
  .await
  .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

  postgres_change_password(postgres.pool(), account.id, &hashed_password)
    .await?;

  redis_revoke_all_sessions(redis, &auth_ctx.user_id, &auth_ctx.token).await?;
  postgres_revoke_all_sessions(
    postgres,
    &auth_ctx.user_id,
    &auth_ctx.session_id,
  )
  .await?;

  Ok(())
}
