use kestrel_common::{
    models::session::{PendingMfa, PendingMfaKind, PendingMfaScope},
    utils::{hasher, totp::TotpSetup},
};
use kestrel_postgres::{
    connection::Database,
    error::DatabaseError,
    operations::account::{get_account_by_id, set_totp_secret},
};
use kestrel_redis::{
    connection::Redis,
    operations::sessions::{create_pending_mfa, delete_pending_mfa, get_pending_mfa},
};
use rocket::{State, post, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::utils::{auth_context::AuthContext, errors::AppError, totp_secret::encrypt_totp_secret};

#[derive(Serialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct EnableTotpResponse {
    pub uri: String,
    pub secret: String,
    pub temp_token: String,
}

#[openapi(tag = "Authentication")]
#[post("/mfa/totp")]
pub async fn enable_totp(
    postgres: &State<Database>,
    redis: &State<Redis>,
    auth_ctx: AuthContext,
) -> Result<Json<EnableTotpResponse>, AppError> {
    let account = match get_account_by_id(postgres, &auth_ctx.user_id).await {
        Ok(acc) => acc,
        Err(e) => match e {
            DatabaseError::NotFound => {
                return Err(AppError::unauthorized("INVALID_CREDENTIALS"));
            }
            other => return Err(AppError::from(other)),
        },
    };

    // Generate a new TOTP configuration and secret
    let totp = TotpSetup::generate();

    // The TOTP secret is stored in Redis, encrypted by the temporary token
    let temp_token = create_pending_mfa(
        redis,
        PendingMfa {
            scope: PendingMfaScope::Setup,
            kind: PendingMfaKind::Totp,
            account_id: account.id.clone(),
            protected_payload: totp.get_secret_base32(),
        },
    )
    .await
    .map_err(|_| AppError::internal_error("PENDING_MFA_STORE_FAILED"))?;

    Ok(Json(EnableTotpResponse {
        uri: totp.build_uri(account.email),
        secret: totp.get_secret_base32(),
        temp_token,
    }))
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct ConfirmEnableTotpRequest {
    pub temp_token: String,
    pub code: String,
    pub password: String,
}

#[openapi(tag = "Authentication")]
#[post("/mfa/totp/confirm", data = "<req>")]
pub async fn confirm_enable_totp(
    postgres: &State<Database>,
    redis: &State<Redis>,
    auth_ctx: AuthContext,
    req: Json<ConfirmEnableTotpRequest>,
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

    let pending_mfa = match get_pending_mfa(redis, &req.temp_token).await {
        Ok(pending_mfa) => pending_mfa,
        Err(_) => return Err(AppError::unauthorized("INVALID_TEMP_TOKEN")),
    };

    let Some(pending_mfa) = pending_mfa else {
        return Err(AppError::unauthorized("INVALID_TEMP_TOKEN"));
    };

    if pending_mfa.scope != PendingMfaScope::Setup || pending_mfa.account_id != auth_ctx.user_id {
        return Err(AppError::unauthorized("INVALID_TEMP_TOKEN"));
    }

    let totp = TotpSetup::from_secret_base32(pending_mfa.protected_payload)
        .map_err(|_| AppError::unauthorized("TOTP_DECRYPT_FAILED"))?;

    // Verify the user's password before allowing MFA enrollment
    hasher::password_verify(req.password.as_bytes(), &account.password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    // Encrypt the TOTP secret using the user's password
    let protected_secret = encrypt_totp_secret(&req.password, &account.password, totp)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    // Persist the secret to the user's account
    set_totp_secret(postgres.pool(), &account.id, Some(&protected_secret))
        .await
        .map_err(AppError::from)?;

    let _ = delete_pending_mfa(redis, &req.temp_token).await;

    Ok(())
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop, schemars::JsonSchema)]
pub struct DisableTotpRequest {
    pub password: String,
}

#[openapi(tag = "Authentication")]
#[delete("/mfa/totp", data = "<req>")]
pub async fn disable_totp(
    postgres: &State<Database>,
    auth_ctx: AuthContext,
    req: Json<DisableTotpRequest>,
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

    // Verify the user's password before allowing removal of MFA
    hasher::password_verify(req.password.as_bytes(), &account.password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    // Remove the TOTP secret from the user's account
    set_totp_secret(postgres.pool(), &account.id, None)
        .await
        .map_err(AppError::from)?;

    Ok(())
}
