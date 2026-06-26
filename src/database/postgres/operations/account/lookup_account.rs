use sqlx::query_as;

use crate::database::postgres::connection::Database;
use crate::database::postgres::error::DatabaseError;
use crate::models::Account;

pub async fn get_account_by_id(
  postgres: &Database,
  id: &str,
) -> Result<Account, DatabaseError> {
  let account = query_as::<_, Account>(
    r#"
        SELECT
            id,
            email,
            email_verified,
            password,
            birthday,
            created_at,
            updated_at,
            totp_secret
        FROM accounts
        WHERE id = $1
        "#,
  )
  .bind(id)
  .fetch_one(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(account)
}

pub async fn get_account_by_email(
  postgres: &Database,
  email: &str,
) -> Result<Account, DatabaseError> {
  let account = query_as::<_, Account>(
    r#"
        SELECT
            id,
            email,
            email_verified,
            password,
            birthday,
            created_at,
            updated_at,
            totp_secret
        FROM accounts
        WHERE email = $1
        "#,
  )
  .bind(email)
  .fetch_one(postgres.pool())
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(account)
}
