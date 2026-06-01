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

  #[error("check constraint violation on '{0}'")]
  CheckViolation(String),

  #[error("invalid operation: {0}")]
  InvalidOperation(String),

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
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown_unique".to_string());

          return Self::UniqueViolation(constraint);
        }
        Some("23503") => {
          return Self::ForeignKeyViolation;
        }
        Some("23502") => {
          return Self::NotNullViolation;
        }
        Some("23514") => {
          let constraint = db_err
            .constraint()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown_check".to_string());

          return Self::CheckViolation(constraint);
        }
        _ => {
          return Self::Other(db_err.message().to_string());
        }
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

impl From<SqlxError> for DatabaseError {
  fn from(err: SqlxError) -> Self {
    Self::from_sqlx(err)
  }
}
