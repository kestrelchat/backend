use sqlx::{PgExecutor, query};

use crate::database::postgres::error::DatabaseError;

pub async fn set_totp_secret(
  postgres: impl PgExecutor<'_>,
  account_id: &str,
  totp_secret: Option<&str>,
) -> Result<(), DatabaseError> {
  query(
    r#"
        UPDATE accounts
        SET totp_secret = $2
        WHERE id = $1
        "#,
  )
  .bind(account_id)
  .bind(totp_secret)
  .execute(postgres)
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
