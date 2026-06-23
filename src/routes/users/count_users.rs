use crate::postgres::connection::Database;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::errors::AppError;

#[openapi(tag = "Core")]
#[get("/count")]
pub async fn count_users(
  db: &rocket::State<Database>,
) -> Result<Json<u64>, AppError> {
  use crate::postgres::operations::user::count_users;
  Ok(Json(count_users(db).await?))
}
