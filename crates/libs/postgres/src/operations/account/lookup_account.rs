// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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
            updated_at
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
            updated_at
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
