use crate::{
  api::guards::auth_context::AuthContext,
  database::postgres::{
    connection::Database, error::DatabaseError,
    operations::relationships::delete_relationship as postgres_delete_relationship,
  },
  errors::AppError,
};
use rocket::State;
use rocket_okapi::openapi;

#[openapi(tag = "Relationships")]
#[delete("/@me/relationships/<target_id>")]
pub async fn delete_relationship(
  postgres: &State<Database>,
  target_id: &str,
  auth_ctx: AuthContext,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;

  postgres_delete_relationship(postgres, user_id.as_str(), target_id)
    .await
    .map_err(|e| match e {
      DatabaseError::InvalidOperation(ref c) => match c.as_str() {
        "RELATIONSHIP_NOT_FOUND" => {
          AppError::not_found("RELATIONSHIP_NOT_FOUND")
        }

        _ => AppError::bad_request("INVALID_OPERATION"),
      },

      other => AppError::from(other),
    })?;

  Ok(())
}
