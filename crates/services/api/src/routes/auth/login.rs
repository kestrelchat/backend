use kestrel_common::{
    hcaptcha::handler::{HCaptchaForm, handle_form},
    models::session::{PendingMfa, PendingMfaKind, PendingMfaScope},
    utils::{
        geoip::GeoIpClient,
        hasher::{self, DECOY_PASSWORD_HASH},
        normalize,
        totp::TotpSetup,
        user_agent::parse_user_agent,
    },
};
use kestrel_config::Config;
use kestrel_postgres::{
    connection::Database,
    error::DatabaseError,
    operations::{
        account::{get_account_by_email, get_account_by_id},
        sessions::{SessionMetadata, create_session as pg_create_session},
    },
};
use kestrel_redis::{
    connection::Redis,
    operations::sessions::{
        create_pending_mfa, create_session as redis_create_session, delete_pending_mfa,
        get_pending_mfa,
    },
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::utils::{
    errors::AppError, request_context::RequestContext, totp_secret::decrypt_totp_secret,
};

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct LoginRequest {
    email: String,
    password: String,
    token: String,
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct MfaLoginRequest {
    temp_token: String,
    code: String,
}

#[derive(Serialize, schemars::JsonSchema)]
pub enum MfaMethod {
    Totp,
}

#[derive(Serialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
#[serde(tag = "status")]
pub enum LoginResponse {
    /// Authentication completed successfully
    Success {
        auth_token: String,
        refresh_token: String,
    },
    /// Password correct, but MFA verification is required
    RequiresMfa {
        temp_token: String,
        #[zeroize(skip)]
        method: MfaMethod,
    },
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
        Ok(acc) => Ok(acc),
        Err(e) => match e {
            DatabaseError::NotFound => Err(AppError::unauthorized("INVALID_CREDENTIALS")),
            other => Err(AppError::from(other)),
        },
    };

    let password = if let Ok(account) = &account {
        &account.password
    } else {
        DECOY_PASSWORD_HASH.as_str()
    };
    hasher::password_verify(req.password.as_bytes(), password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    let account = account?;

    let Some(totp_secret) = account.totp_secret else {
        let response = establish_session(postgres, redis, geoip, &ctx, &account.id).await?;
        return Ok(Json(response));
    };

    // Decrypt the TOTP secret using the user's password
    let totp = decrypt_totp_secret(&req.password, &account.password, totp_secret)
        .await
        .map_err(|_| AppError::unauthorized("TOTP_DECRYPT_FAILED"))?;

    // The TOTP secret is stored in Redis, encrypted by the temporary token
    let temp_token = create_pending_mfa(
        redis,
        PendingMfa {
            scope: PendingMfaScope::Login,
            kind: PendingMfaKind::Totp,
            account_id: account.id.clone(),
            protected_payload: totp.get_secret_base32(),
        },
    )
    .await
    .map_err(|_| AppError::internal_error("PENDING_MFA_STORE_FAILED"))?;

    Ok(Json(LoginResponse::RequiresMfa {
        temp_token: temp_token.to_string(),
        method: MfaMethod::Totp,
    }))
}

#[openapi(tag = "Authentication")]
#[post("/login/mfa", data = "<req>")]
pub async fn login_mfa(
    postgres: &State<Database>,
    redis: &State<Redis>,
    geoip: &State<GeoIpClient>,
    ctx: RequestContext,
    req: Json<MfaLoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let pending_mfa = get_pending_mfa(redis, &req.temp_token)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_MFA_TOKEN"))?
        .ok_or_else(|| AppError::unauthorized("EXPIRED_MFA_TOKEN"))?;

    if pending_mfa.scope != PendingMfaScope::Login {
        return Err(AppError::unauthorized("INVALID_MFA_TOKEN"));
    }

    let account = get_account_by_id(postgres, &pending_mfa.account_id)
        .await
        .map_err(AppError::from)?;

    match pending_mfa.kind {
        PendingMfaKind::Totp => {
            let totp = TotpSetup::from_secret_base32(pending_mfa.protected_payload)
                .map_err(|_| AppError::internal_error("INVALID_MFA_SECRET"))?;

            if totp.verify(&req.code).is_err() {
                return Err(AppError::unauthorized("INVALID_MFA_CODE"));
            }
        }
    }

    let _ = delete_pending_mfa(redis, &req.temp_token).await;

    let response = establish_session(postgres, redis, geoip, &ctx, &account.id).await?;
    Ok(Json(response))
}

/// Dispatches session state initialization across PostgreSQL and Redis.
async fn establish_session(
    postgres: &Database,
    redis: &Redis,
    geoip: &GeoIpClient,
    ctx: &RequestContext,
    account_id: &str,
) -> Result<LoginResponse, AppError> {
    let ip = ctx.ip.ok_or(AppError::unauthorized("MISSING_IP"))?;
    let user_agent = ctx.user_agent.as_deref().unwrap_or("Unknown");

    let geo = geoip.lookup(ip).await.unwrap_or_default();
    let ua = parse_user_agent(user_agent);

    let country = geo.country.unwrap_or_else(|| "Unknown".to_string());
    let region = geo.region.unwrap_or_else(|| "Unknown".to_string());
    let city = geo.city.unwrap_or_else(|| "Unknown".to_string());

    let operating_system = ua.os_family;
    let platform = ua.browser_family;

    let pg_session = pg_create_session(
        postgres,
        account_id,
        SessionMetadata {
            ip_address: Some(ip),
            country: Some(country),
            region: Some(region),
            city: Some(city),
            user_agent: Some(user_agent.to_string()),
            operating_system: Some(operating_system),
            platform: Some(platform),
        },
    )
    .await
    .map_err(AppError::from)?;

    let auth_token = redis_create_session(redis, &pg_session.session.id, account_id)
        .await
        .map_err(|_| AppError::internal_error("SESSION_STORE_FAILED"))?;

    Ok(LoginResponse::Success {
        auth_token,
        refresh_token: pg_session.refresh_token,
    })
}
