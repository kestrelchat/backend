use kestrel_common::utils::{hasher, totp::TotpSetup};
use kestrel_postgres::{
    connection::Database,
    error::DatabaseError,
    operations::account::{get_account_by_id, set_totp_secret},
};
use rocket::{State, post, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};

use crate::utils::{auth_context::AuthContext, errors::AppError, totp_secret::encrypt_totp_secret};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct SetupTotpRequest {
    pub password: String,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct SetupTotpResponse {
    pub uri: String,
    pub secret: String,
}

#[openapi(tag = "Authentication")]
#[post("/mfa/totp", data = "<req>")]
pub async fn setup_totp(
    postgres: &State<Database>,
    auth_ctx: AuthContext,
    req: Json<SetupTotpRequest>,
) -> Result<Json<SetupTotpResponse>, AppError> {
    let account = match get_account_by_id(postgres, &auth_ctx.user_id).await {
        Ok(acc) => acc,
        Err(e) => match e {
            DatabaseError::NotFound => {
                return Err(AppError::unauthorized("INVALID_CREDENTIALS"));
            }
            other => return Err(AppError::from(other)),
        },
    };

    // Verify the user's password before allowing MFA enrollment
    hasher::password_verify(req.password.as_bytes(), &account.password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    // Generate a new TOTP configuration and secret
    let totp = TotpSetup::generate();
    let secret_base32 = totp.get_secret_base32();

    // Generate a URI for authenticator applications
    let uri = totp.build_uri(account.email);

    // Encrypt the TOTP secret using the user's password
    let protected_secret = encrypt_totp_secret(&req.password, &account.password, totp)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    // Persist the secret to the user's account
    set_totp_secret(postgres, &account.id, Some(&protected_secret))
        .await
        .map_err(AppError::from)?;

    Ok(Json(SetupTotpResponse {
        uri,
        secret: secret_base32,
    }))
}
