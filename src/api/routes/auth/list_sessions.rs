use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    operations::sessions::lookup_sessions::lookup_sessions,
  },
  errors::AppError,
  models::Session,
};
use chrono::{DateTime, Utc};
use rocket::{State, get, serde::json::Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct SessionResponse {
  pub sessions: Vec<SessionView>,
}

#[derive(Serialize, JsonSchema)]
pub struct SessionView {
  pub id: String,

  pub country: Option<String>,
  pub region: Option<String>,
  pub city: Option<String>,

  pub operating_system: Option<String>,
  pub platform: Option<String>,

  pub last_used_at: DateTime<Utc>,
}

#[openapi(tag = "Sessions")]
#[get("/sessions")]
pub async fn list_sessions(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
) -> Result<Json<SessionResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  let sessions = lookup_sessions(postgres, &user_id)
    .await
    .map_err(AppError::from)?
    .into_iter()
    .map(|s: Session| SessionView {
      id: s.id,

      country: s.country,
      region: s.region,
      city: s.city,

      operating_system: s.operating_system,
      platform: s.platform,

      last_used_at: s.last_used_at,
    })
    .collect::<Vec<_>>();

  Ok(Json(SessionResponse { sessions }))
}
