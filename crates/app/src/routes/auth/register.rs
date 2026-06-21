use chrono::{Datelike, NaiveDate, Utc};
use kestrel_common::{
  hcaptcha::handler::{HCaptchaForm, handle_form},
  utils::{
    hasher, normalize,
    validation::{ValidationError, email, password, username},
  },
};
use kestrel_config::Config;
use kestrel_postgres::{
  connection::Database,
  error::DatabaseError,
  operations::{account::create_account, user::create_user},
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{guards::rate_limit::WithinRateLimit, utils::errors::AppError};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct RegisterRequest {
  email: String,
  username: String,
  password: String,
  #[zeroize(skip)]
  birthday: Option<NaiveDate>,
  hcaptcha_token: Option<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct RegisterResponse {
  id: String,
  email: String,
}

#[openapi(tag = "Authentication")]
#[post("/register", data = "<req>")]
pub async fn register(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  config: &State<Config>,
  req: Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
  if !config.features.registration.enabled {
    return Err(AppError::unauthorized("REGISTRATION_DISABLED"));
  }

  if config.features.hcaptcha.enabled {
    let token = req
      .hcaptcha_token
      .as_deref()
      .ok_or_else(|| AppError::unauthorized("MISSING_CAPTCHA"))?;

    handle_form(
      HCaptchaForm { token },
      config.features.hcaptcha.secret.as_deref(),
    )
    .await
    .map_err(|_| AppError::unauthorized("FAILED_CAPTCHA"))?;
  }

  let normalized_email = normalize::identity(&req.email);
  let normalized_username = normalize::identity(&req.username);

  email::validate(&normalized_email, config.is_production)
    .await
    .map_err(ValidationError::Email)?;

  password::validate(&req.password)
    .await
    .map_err(ValidationError::Password)?;

  username::validate(&normalized_username)
    .await
    .map_err(ValidationError::Username)?;

  let birthday = req
    .birthday
    .ok_or(AppError::bad_request("BIRTHDAY_EMPTY"))?;
  if !is_old_enough(birthday, config.features.registration.minimum_age as i32) {
    return Err(AppError::bad_request("AGE_TOO_YOUNG"));
  }

  let hashed_password =
    hasher::password_hash(Zeroizing::new(req.password.as_bytes().to_vec()))
      .await
      .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

  let mut tx = postgres
    .pool()
    .begin()
    .await
    .map_err(|_| AppError::internal_error("BEGIN_TX_FAILED"))?;

  let account =
    create_account(tx.as_mut(), &normalized_email, &hashed_password, birthday)
      .await
      .map_err(|e| match e {
        DatabaseError::UniqueViolation(ref c) if c == "accounts_email_key" => {
          AppError::conflict("EMAIL_TAKEN")
        }
        other => AppError::from(other),
      })?;

  // will be used once email verification is implemented
  let _user =
    create_user(tx.as_mut(), account.id.clone(), &normalized_username)
      .await
      .map_err(|e| match e {
        DatabaseError::UniqueViolation(ref c) if c == "user_unique_tag" => {
          AppError::conflict("USERNAME_TAKEN")
        }
        other => AppError::from(other),
      })?;

  tx.commit()
    .await
    .map_err(|_| AppError::internal_error("COMMIT_TX_FAILED"))?;

  // TODO: SEND VERIFICATION EMAIL

  Ok(Json(RegisterResponse {
    id: account.id,
    email: account.email,
  }))
}

fn is_old_enough(birthday: NaiveDate, min_age: i32) -> bool {
  let today = Utc::now().date_naive();

  let age = today.year()
    - birthday.year()
    - if (today.month(), today.day()) < (birthday.month(), birthday.day()) {
      1
    } else {
      0
    };

  age >= min_age
}
