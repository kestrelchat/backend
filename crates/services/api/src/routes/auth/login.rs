// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use kestrel_common::utils::{geoip::GeoIpClient, hasher, normalize, user_agent::parse_user_agent};
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
    ctx: RequestContext,
    req: Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
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

    hasher::verify(req.password.as_bytes(), &account.password)
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
