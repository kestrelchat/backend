use kestrel_common::utils::{
  hasher,
  validation::{ValidationError, email, password},
};
use kestrel_config::Config;
use kestrel_postgres::{
  connection::Database,
  operations::account::{
    change_password, get_account_by_email, set_totp_secret,
  },
};
use kestrel_redis::{
  connection::Redis,
  operations::password::{check_reset_token, create_reset_token},
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::errors::AppError;

#[derive(Deserialize, JsonSchema)]
pub struct PasswordResetRequest {
  pub email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct PasswordResetReqResponse {
  pub reset_token: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct PasswordResetValidateRequest {
  pub token: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ResetPasswordRequest {
  pub token: String,
  pub new_password: String,
}

#[openapi(tag = "Authentication")]
#[post("/password/reset/request", data = "<req>")]
pub async fn request_password_reset(
  postgres: &State<Database>,
  redis: &State<Redis>,
  config: &State<Config>,
  req: Json<PasswordResetRequest>,
) -> Result<Json<PasswordResetReqResponse>, AppError> {
  email::validate(&req.email, config.is_production)
    .await
    .map_err(|_| AppError::bad_request("INVALID_EMAIL"))?;

  let mut reset_token = "NOP".to_string(); // ASSEMBLY REFRENCE!!111!

  if let Ok(account) = get_account_by_email(postgres, &req.email).await
    && let Ok(token) = create_reset_token(redis, &account.id).await
  {
    reset_token = token;

    // SEND EMAIL HERE
  }

  // We don't have a tool for sending emails, this is temporary.
  // We will return OK in the future.
  Ok(Json(PasswordResetReqResponse { reset_token }))
}

#[openapi(tag = "Authentication")]
#[post("/password/reset/validate", data = "<req>")]
pub async fn validate_password_reset(
  redis: &State<Redis>,
  req: Json<PasswordResetValidateRequest>,
) -> Result<(), AppError> {
  check_reset_token(redis, &req.token)
    .await
    .map_err(|_| AppError::bad_request("INVALID_TOKEN"))?;

  Ok(())
}

#[openapi(tag = "Authentication")]
#[post("/password/reset", data = "<req>")]
pub async fn reset_password(
  postgres: &State<Database>,
  redis: &State<Redis>,
  req: Json<ResetPasswordRequest>,
) -> Result<(), AppError> {
  password::validate(&req.new_password)
    .await
    .map_err(ValidationError::Password)?;

  let account_id = check_reset_token(redis.inner(), &req.token)
    .await
    .map_err(|_| AppError::unauthorized("INVALID_TOKEN"))?;

  let hashed_password = hasher::password_hash(req.new_password.as_bytes())
    .await
    .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

  let mut tx = postgres
    .pool()
    .begin()
    .await
    .map_err(|_| AppError::internal_error("DB_TX_FAILED"))?;

  set_totp_secret(postgres.pool(), &account_id, None)
    .await
    .map_err(AppError::from)?;

  change_password(tx.as_mut(), account_id, &hashed_password).await?;

  tx.commit()
    .await
    .map_err(|_| AppError::internal_error("COMMIT_TX_FAILED"))?;

  Ok(())
}
