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

use chrono::{NaiveDate, Utc};
use sqlx::query_as;
use ulid::Ulid;

use crate::connection::Database;
use crate::error::DatabaseError;
use kestrel_common::models::Account;

pub async fn create_account(
    db: &Database,
    email: &str,
    password: &str,
    birthday: NaiveDate,
) -> Result<Account, DatabaseError> {
    let id = Ulid::new().to_string();
    let created_at = Utc::now();
    let updated_at = created_at;

    let account = query_as::<_, Account>(
        r#"
        INSERT INTO accounts (id, email, password, birthday, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, email_verified, password, birthday, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password)
    .bind(birthday)
    .bind(created_at)
    .bind(updated_at)
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(account)
}
