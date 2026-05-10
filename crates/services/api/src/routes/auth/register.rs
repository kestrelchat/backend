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
use kestrel_common::utils::{
    hasher, normalize,
    validation::{ValidationError, email, password, username},
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
    postgres: &State<Database>,
    config: &State<Config>,
    req: Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
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
    if !is_old_enough(birthday, config.api.registration.minimum_age as i32) {
        return Err(AppError::bad_request("AGE_TOO_YOUNG"));
    }

    let hashed_password = hasher::hash(req.password.as_bytes())
        .await
        .map_err(|_| AppError::internal_error("HASH_FAILED"))?;

    let account = create_account(postgres, &normalized_email, &hashed_password, birthday)
        .await
        .map_err(|e| match e {
            DatabaseError::UniqueViolation(ref c) if c == "accounts_email_key" => {
                AppError::conflict("EMAIL_TAKEN")
            }
            other => AppError::from(other),
        })?;

    let _user = create_user(postgres, account.id.clone(), &normalized_username)
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
