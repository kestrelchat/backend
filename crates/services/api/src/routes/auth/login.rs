use kestrel_common::{
    hcaptcha::handler::{HCaptchaForm, handle_form},
    utils::{geoip::GeoIpClient, hasher, normalize, user_agent::parse_user_agent},
};
use kestrel_config::Config;
use kestrel_postgres::{
    connection::Database,
    error::DatabaseError,
    operations::{
        account::get_account_by_email,
        sessions::{SessionMetadata, create_session as pg_create_session},
    },
};
use kestrel_redis::{
    connection::Redis, operations::sessions::create_session as redis_create_session,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};

use crate::utils::{errors::AppError, request_context::RequestContext};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct LoginRequest {
    email: String,
    password: String,
    token: String,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct LoginResponse {
    auth_token: String,
    refresh_token: String,
}

#[openapi(tag = "Authentication")]
#[post("/login", data = "<req>")]
pub async fn login(
    postgres: &State<Database>,
    redis: &State<Redis>,
    geoip: &State<GeoIpClient>,
    config: &State<Config>,
    ctx: RequestContext,
    req: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if config.features.hcaptcha.enabled
        && let Err(_) = handle_form(
            HCaptchaForm { token: &req.token },
            config.features.hcaptcha.secret.as_deref(),
        )
        .await
    {
        return Err(AppError::unauthorized("FAILED_CAPTCHA"));
    }

    let normalized_email = normalize::identity(&req.email);

    let account = match get_account_by_email(postgres, &normalized_email).await {
        Ok(acc) => acc,

        Err(e) => match e {
            DatabaseError::NotFound => {
                return Err(AppError::unauthorized("INVALID_CREDENTIALS"));
            }

            other => return Err(AppError::from(other)),
        },
    };

    hasher::password_verify(req.password.as_bytes(), &account.password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    let ip = ctx.ip.ok_or(AppError::unauthorized("MISSING_IP"))?;
    let user_agent = ctx.user_agent.unwrap_or_else(|| "Unknown".to_string());

    let geo = geoip.lookup(ip).await.unwrap_or_default();
    let ua = parse_user_agent(&user_agent);

    let country = geo.country.unwrap_or_else(|| "Unknown".to_string());
    let region = geo.region.unwrap_or_else(|| "Unknown".to_string());
    let city = geo.city.unwrap_or_else(|| "Unknown".to_string());

    let operating_system = ua.os_family;
    let platform = ua.browser_family;

    let pg_session = pg_create_session(
        postgres,
        &account.id,
        SessionMetadata {
            ip_address: Some(ip),
            country: Some(country),
            region: Some(region),
            city: Some(city),
            user_agent: Some(user_agent),
            operating_system: Some(operating_system),
            platform: Some(platform),
        },
    )
    .await
    .map_err(AppError::from)?;

    let auth_token = redis_create_session(redis, &pg_session.session.id, &account.id)
        .await
        .map_err(|_| AppError::internal_error("SESSION_STORE_FAILED"))?;

    Ok(Json(LoginResponse {
        auth_token,
        refresh_token: pg_session.refresh_token,
    }))
}
