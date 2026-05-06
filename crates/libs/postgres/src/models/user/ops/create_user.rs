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
use sqlx::query_as;

use crate::{connection::Database, error::DatabaseError, models::user::User};

pub async fn create_user(db: &Database, id: String, username: &str) -> Result<User, DatabaseError> {
    let created_at = Utc::now();
    let updated_at = created_at;

    let discrim = "0000";

    let user = query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, discrim, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, discrim, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(username)
    .bind(discrim)
    .bind(created_at)
    .bind(updated_at)
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(user)
}
