/*
 * Kestrel - a modern instant-messaging service written in Rust
 * Copyright (C) 2026 Kestrel Chat
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use kestrel_config::Config;
use kestrel_postgres::connection::Database;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::utils::errors::AppError;

#[derive(Serialize, JsonSchema)]
pub struct Meta {
    pub kestrel: String,
    pub features: FeaturesMeta,
}

#[derive(Serialize, JsonSchema)]
pub struct FeaturesMeta {
    pub registration: RegistrationMeta,
}

#[derive(Serialize, JsonSchema)]
pub struct RegistrationMeta {
    pub minimum_age: u32,
}

#[openapi(tag = "Core")]
#[get("/")]
pub fn meta(config: &rocket::State<Config>) -> Json<Meta> {
    Json(Meta {
        kestrel: env!("CARGO_PKG_VERSION").into(),
        features: FeaturesMeta {
            registration: RegistrationMeta {
                minimum_age: config.api.registration.minimum_age,
            },
        },
    })
}

#[openapi(tag = "Core")]
#[get("/users/count")]
pub async fn users_count(db: &rocket::State<Database>) -> Result<Json<u64>, AppError> {
    use kestrel_postgres::operations::user::count_users;
    Ok(Json(count_users(db).await?))
}
