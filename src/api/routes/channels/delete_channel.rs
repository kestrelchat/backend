use rocket::State;
use rocket_okapi::openapi;

use crate::{
  api::guards::{auth_context::AuthContext, rate_limit::WithinRateLimit},
  database::postgres::{
    connection::Database,
    operations::channels::delete_channel::delete_channel as postgres_delete_channel,
  },
  errors::AppError,
};

#[openapi(tag = "Channels")]
#[delete("/<channel_id>")]
pub async fn delete_channel(
  _within_rate_limit: WithinRateLimit,
  postgres: &State<Database>,
  auth_ctx: AuthContext,
  channel_id: &str,
) -> Result<(), AppError> {
  let user_id = auth_ctx.user_id;

  postgres_delete_channel(postgres, &user_id, channel_id).await?;

  Ok(())
}
