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

use sqlx::{Error as SqlxError, migrate::MigrateError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("connection error: {0}")]
    Connection(SqlxError),

    #[error("query error: {0}")]
    Query(SqlxError),

    #[error("migration failed: {0}")]
    Migration(#[from] MigrateError),

    #[error("record not found")]
    NotFound,

    #[error("unique constraint violation on '{0}'")]
    UniqueViolation(String),

    #[error("foreign key constraint violation")]
    ForeignKeyViolation,

    #[error("not-null constraint violation")]
    NotNullViolation,

    #[error("check constraint violation")]
    CheckViolation,

    #[error("other database error: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn from_sqlx(err: SqlxError) -> Self {
        if let SqlxError::Database(db_err) = &err {
            match db_err.code().as_deref() {
                Some("23505") => {
                    let constraint = db_err
                        .constraint()
                        .map(|c| c.into())
                        .unwrap_or_else(|| "unknown".into());
                    return Self::UniqueViolation(constraint);
                }
                Some("23503") => return Self::ForeignKeyViolation,
                Some("23502") => return Self::NotNullViolation,
                Some("23514") => return Self::CheckViolation,
                _ => return Self::Other(db_err.message().to_string()),
            }
        }

        match err {
            SqlxError::RowNotFound => Self::NotFound,
            SqlxError::PoolTimedOut | SqlxError::PoolClosed | SqlxError::Io(_) => {
                Self::Connection(err)
            }
            _ => Self::Query(err),
        }
    }
}
