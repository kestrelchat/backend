use kestrel_common::models::RelationshipAction;
use kestrel_postgres::{
  connection::Database, error::DatabaseError,
  operations::relationships::create_relationship as pg_create_relationship,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use serde::Deserialize;

use crate::utils::{auth_context::AuthContext, errors::AppError};

#[derive(Deserialize, schemars::JsonSchema)]
pub struct CreateRelationship {
  #[serde(rename = "action")]
  pub relationship_action: RelationshipAction,
}

#[openapi(tag = "Relationships")]
#[post("/@me/relationships/<target_id>", data = "<req>")]
pub async fn create_relationship(
  postgres: &State<Database>,
  target_id: &str,
  auth_ctx: AuthContext,
  req: Json<CreateRelationship>,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;

  pg_create_relationship(
    postgres,
    user_id.as_str(),
    target_id,
    req.relationship_action.clone(),
  )
  .await
  .map_err(|e| match e {
    DatabaseError::InvalidOperation(ref c) => match c.as_str() {
      "REQUEST_ALREADY_SENT" => AppError::conflict("REQUEST_ALREADY_SENT"),

      "ALREADY_FRIENDS" => AppError::conflict("ALREADY_FRIENDS"),

      "RELATIONSHIP_FAILED" => AppError::bad_request("RELATIONSHIP_FAILED"),

      _ => AppError::bad_request("INVALID_OPERATION"),
    },

    DatabaseError::CheckViolation(ref c) => match c.as_str() {
      "no_self_relation" => AppError::bad_request("CANNOT_RELATE_TO_SELF"),
      "nickname_only_for_friends" => {
        AppError::bad_request("INVALID_NICKNAME_USAGE")
      }
      "nickname_length" => AppError::bad_request("NICKNAME_TOO_LONG"),
      _ => AppError::bad_request("RELATIONSHIP_CONSTRAINT_VIOLATION"),
    },

    DatabaseError::UniqueViolation(ref c) => match c.as_str() {
      "relationships_pkey" => AppError::conflict("RELATIONSHIP_ALREADY_EXISTS"),
      _ => AppError::conflict("ALREADY_EXISTS"),
    },

    other => AppError::from(other),
  })?;

  Ok(())
}
