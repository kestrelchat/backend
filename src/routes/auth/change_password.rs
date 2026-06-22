use kestrel_common::utils::{
  hasher,
  validation::{ValidationError, password},
};
use kestrel_postgres::{
  connection::Database,
  error::DatabaseError,
  operations::{
    account::{
      change_password as postgres_change_password, get_account_by_id,
      set_totp_secret,
    },
    sessions::revoke_all_sessions as postgres_revoke_all_sessions,
  },
};
use kestrel_redis::{
  connection::Redis,
  operations::sessions::revoke_all_sessions as redis_revoke_all_sessions,
};

use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{
  guards::rate_limit::WithinRateLimit,
  utils::{
    auth_context::AuthContext,
    errors::AppError,
    totp_secret::{decrypt_totp_secret, encrypt_totp_secret},
  },
};

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

  password::validate(&req.new_password)
    .await
    .map_err(ValidationError::Password)?;

  let hashed_password =
    hasher::password_hash(Zeroizing::new(req.new_password.as_bytes().to_vec()))
      .await
      .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

  let mut tx = postgres
    .pool()
    .begin()
    .await
    .map_err(|_| AppError::internal_error("DB_TX_FAILED"))?;

  if let Some(old_ciphertext) = account.totp_secret {
    let totp =
      decrypt_totp_secret(&req.old_password, &account.password, old_ciphertext)
        .await
        .map_err(|_| AppError::internal_error("TOTP_DECRYPT_FAILED"))?;
    let new_ciphertext =
      encrypt_totp_secret(&req.new_password, &hashed_password, totp)
        .await
        .map_err(|_| AppError::internal_error("TOTP_ENCRYPT_FAILED"))?;
    set_totp_secret(tx.as_mut(), &account.id, Some(&new_ciphertext)).await?;
  }
  postgres_change_password(tx.as_mut(), account.id, &hashed_password).await?;

  tx.commit()
    .await
    .map_err(|_| AppError::internal_error("DB_TX_FAILED"))?;

  redis_revoke_all_sessions(redis, &auth_ctx.user_id, &auth_ctx.token).await?;
  postgres_revoke_all_sessions(
    postgres,
    &auth_ctx.user_id,
    &auth_ctx.session_id,
  )
  .await?;

  Ok(())
}
