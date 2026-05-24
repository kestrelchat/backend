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

use chrono::Utc;
use sqlx::query;

use crate::connection::Database;
use crate::error::DatabaseError;

pub async fn change_password(
    db: &Database,
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
    .execute(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(())
}
