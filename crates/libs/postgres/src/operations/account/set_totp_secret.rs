use sqlx::query;

use crate::connection::Database;
use crate::error::DatabaseError;

pub async fn set_totp_secret(
    db: &Database,
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
    .execute(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(())
}
