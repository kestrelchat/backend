use sqlx::query_as;

use crate::connection::Database;
use crate::error::DatabaseError;
use kestrel_common::models::Account;

pub async fn get_account_by_id(db: &Database, id: &str) -> Result<Account, DatabaseError> {
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
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(account)
}

pub async fn get_account_by_email(db: &Database, email: &str) -> Result<Account, DatabaseError> {
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
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(account)
}
