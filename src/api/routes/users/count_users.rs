use crate::database::postgres::connection::Database;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::errors::AppError;

#[openapi(tag = "Core")]
#[get("/count")]
pub async fn count_users(
  postgres: &rocket::State<Database>,
) -> Result<Json<u64>, AppError> {
  use crate::database::postgres::operations::user::count_users;
  Ok(Json(count_users(postgres).await?))
}
