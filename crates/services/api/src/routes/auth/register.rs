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

use chrono::{Datelike, NaiveDate, Utc};
use common::utils::{
    normalize,
    validation::{ValidationError, email, password},
};
use config::Config;
use database::{
    connection::Database,
    error::DatabaseError,
    models::{
        account::{AccountOps, AccountRepository},
        user::{UserOps, UserRepository},
    },
};
use rocket::{State, serde::json::Json};
use rocket_okapi::{okapi::schemars, openapi};
use serde::{Deserialize, Serialize};

use crate::utils::errors::AppError;

#[derive(Deserialize, schemars::JsonSchema)]
pub struct RegisterRequest {
    email: String,
    username: String,
    password: String,
    birthday: Option<NaiveDate>,
}

#[derive(Serialize, schemars::JsonSchema)]
pub struct RegisterResponse {
    id: String,
    email: String,
}

#[openapi(tag = "Authentication")]
#[post("/register", data = "<req>")]
pub async fn register(
    db: &State<Database>,
    config: &State<Config>,
    req: Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let normalized_email = normalize::identity(&req.email);

    email::validate(&normalized_email, config.is_production)
        .await
        .map_err(ValidationError::Email)?;

    password::validate(&req.password)
        .await
        .map_err(ValidationError::Password)?;

    let birthday = req
        .birthday
        .ok_or(AppError::bad_request("BIRTHDAY_EMPTY"))?;
    if !is_old_enough(birthday, config.api.registration.minimum_age as i32) {
        return Err(AppError::bad_request("AGE_TOO_YOUNG"));
    }

    let hashed_password = common::utils::hasher::hash(req.password.as_bytes())
        .await
        .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

    let account = AccountRepository
        .create_account(db, &normalized_email, &hashed_password, birthday)
        .await
        .map_err(|e| match e {
            DatabaseError::UniqueViolation(ref c) if c == "accounts_email_key" => {
                AppError::conflict("EMAIL_TAKEN")
            }
            other => AppError::from(other),
        })?;

    let _user = UserRepository
        .create_user(db, account.id.clone(), req.username.as_str())
        .await
        .map_err(|e| match e {
            DatabaseError::UniqueViolation(ref c) if c == "user_unique_tag" => {
                AppError::conflict("USERNAME_TAKEN")
            }
            other => AppError::from(other),
        })?;

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
        - if today.ordinal() < birthday.ordinal() {
            1
        } else {
            0
        };

    age >= min_age
}
