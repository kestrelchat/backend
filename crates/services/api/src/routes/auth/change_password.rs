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

use kestrel_common::utils::{
    hasher, normalize,
    validation::{ValidationError, password},
};
use kestrel_postgres::{
    connection::Database,
    error::DatabaseError,
    operations::account::{change_password as postgres_change_password, get_account_by_email},
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::Deserialize;

use crate::utils::errors::AppError;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct ChangePasswordRequest {
    email: String,
    old_password: String,
    new_password: String,
}

#[openapi(tag = "Authentication")]
#[post("/password/change", data = "<req>")]
pub async fn change_password(
    postgres: &State<Database>,
    req: Json<ChangePasswordRequest>,
) -> Result<(), AppError> {
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

    hasher::verify(req.old_password.as_bytes(), &account.password)
        .await
        .map_err(|_| AppError::unauthorized("INVALID_CREDENTIALS"))?;

    password::validate(&req.new_password)
        .await
        .map_err(ValidationError::Password)?;

    let hashed_password = hasher::hash(req.new_password.as_bytes())
        .await
        .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

    postgres_change_password(postgres, account.id, &hashed_password).await?;

    Ok(())
}
