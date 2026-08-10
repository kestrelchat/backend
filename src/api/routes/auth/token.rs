use crate::{
  adapters::{
    crypto::hasher::{self, DECOY_PASSWORD_HASH},
    geoip::GeoIpClient,
    hcaptcha::handler::{HCaptchaForm, handle_form},
    totp::TotpSetup,
    user_agent::parse_user_agent,
  },
  api::guards::{rate_limit::WithinRateLimit, request_context::RequestContext},
  config::Config,
  database::{
    postgres::{
      connection::Database,
      error::DatabaseError,
      operations::{
        account::{get_account_by_email, get_account_by_id},
        sessions::{
          SessionMetadata, create_session as postgres_create_session,
          get_session,
        },
      },
    },
    redis::{
      connection::Redis,
      operations::sessions::{
        create_pending_mfa, create_session as redis_create_session,
        delete_pending_mfa, get_pending_mfa,
      },
    },
  },
  errors::AppError,
  models::{
    ValidationError,
    session::{PendingMfa, PendingMfaKind, PendingMfaScope},
    user::email::Email,
  },
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct TokenRequest {
  refresh_token: String,
}

#[derive(Serialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
#[serde(tag = "status")]
pub enum TokenResponse {
  Success {
    auth_token: String,
    refresh_token: String,
  },
}

#[openapi(tag = "Authentication")]
#[post("/token", data = "<req>")]
pub async fn token(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  redis: &State<Redis>,
  req: Json<TokenRequest>,
) -> Result<Json<TokenResponse>, AppError> {
  let refresh_token = &req.refresh_token;
  let session = get_session(postgres, refresh_token)
    .await
    .map_err(AppError::from)?;
  let session_id = &session[0].id;
  let account_id = &session[0].user_id;
  let auth_token = redis_create_session(redis, session_id, account_id)
    .await
    .map_err(|_| AppError::internal_error("SESSION_STORE_FAILED"))?;

  Ok(Json(TokenResponse::Success {
    auth_token,
    refresh_token: refresh_token.into(),
  }))
}
