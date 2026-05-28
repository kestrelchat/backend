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
                minimum_age: config.features.registration.minimum_age,
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
