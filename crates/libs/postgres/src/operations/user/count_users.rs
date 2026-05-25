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

use sqlx::{Row, query};

use crate::{connection::Database, error::DatabaseError};

/// Returns the estimated count of users using pg_class.
///
/// If the estimated count query yields a number not greater than 0,
/// the method falls back to using an exact `COUNT(*)` query and if
/// the actual count is non-zero, issues an analysis of the table.
///
/// For more information on how the estimate works, see
/// [PostgreSQL Wiki](<https://wiki.postgresql.org/wiki/Count_estimate>).
pub async fn count_users(db: &Database) -> Result<u64, DatabaseError> {
    let estimate_row = query(
        r#"
        SELECT reltuples::bigint AS estimate
        FROM pg_class
        WHERE relname = 'users'
        "#,
    )
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    let estimate: i64 = estimate_row.get("estimate");

    if estimate > 0 {
        return Ok(estimate.cast_unsigned());
    }

    let exact_row = query("SELECT COUNT(*) AS exact FROM users")
        .fetch_one(db.pool())
        .await
        .map_err(DatabaseError::from_sqlx)?;

    let exact: i64 = exact_row.get("exact");

    if exact != 0 {
        query("ANALYZE users")
            .execute(db.pool())
            .await
            .map_err(DatabaseError::from_sqlx)?;
    }

    Ok(exact.cast_unsigned())
}
