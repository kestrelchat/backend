use chrono::{NaiveDate, Utc};
use sqlx::{PgExecutor, query_as};
use ulid::Ulid;

use crate::database::postgres::error::DatabaseError;
use crate::models::Account;

pub async fn create_account(
  postgres: impl PgExecutor<'_>,
  email: &str,
  password: &str,
  birthday: NaiveDate,
) -> Result<Account, DatabaseError> {
  let id = Ulid::new().to_string();
  let created_at = Utc::now();
  let updated_at = created_at;

  let account = query_as::<_, Account>(
        r#"
        INSERT INTO accounts (id, email, password, birthday, created_at, updated_at, totp_secret)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, email, email_verified, password, birthday, created_at, updated_at, totp_secret
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password)
    .bind(birthday)
    .bind(created_at)
    .bind(updated_at)
    .bind(None::<String>)
    .fetch_one(postgres)
    .await
    .map_err(DatabaseError::from_sqlx)?;

  Ok(account)
}
