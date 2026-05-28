use chrono::Utc;
use sqlx::{PgExecutor, query};

use crate::error::DatabaseError;

pub async fn change_password(
  db: impl PgExecutor<'_>,
  id: String,
  password: &str,
) -> Result<(), DatabaseError> {
  let updated_at = Utc::now();

  query(
    r#"
        UPDATE accounts
        SET
            password = $1,
            updated_at = $2
        WHERE
            id = $3
        "#,
  )
  .bind(password)
  .bind(updated_at)
  .bind(id)
  .execute(db)
  .await
  .map_err(DatabaseError::from_sqlx)?;

  Ok(())
}
