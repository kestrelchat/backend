use crate::database::postgres::{connection::Database, error::DatabaseError};
use crate::models::{Relationship, RelationshipType};

pub async fn delete_relationship(
  postgres: &Database,
  user_id: &str,
  target_id: &str,
) -> Result<Vec<Relationship>, DatabaseError> {
  let relationship_type: Option<RelationshipType> = sqlx::query_scalar(
    r#"
      SELECT type
      FROM relationships
      WHERE user_id = $1
        AND target_id = $2
      LIMIT 1
    "#,
  )
  .bind(user_id)
  .bind(target_id)
  .fetch_optional(postgres.pool())
  .await?;

  let relationship_type = relationship_type.ok_or_else(|| {
    DatabaseError::InvalidOperation("RELATIONSHIP_NOT_FOUND".to_string())
  })?;

  match relationship_type {
    RelationshipType::Friend => {
      let relationships = sqlx::query_as::<_, Relationship>(
        r#"
          DELETE FROM relationships
          WHERE (user_id, target_id, type) IN (
            ($1, $2, 'friend'),
            ($2, $1, 'friend')
          )
          RETURNING user_id, target_id, type, nickname, created_at, updated_at
        "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_all(postgres.pool())
      .await?;

      Ok(relationships)
    }

    RelationshipType::IncomingRequest | RelationshipType::OutgoingRequest => {
      let relationships = sqlx::query_as::<_, Relationship>(
        r#"
          DELETE FROM relationships
          WHERE (user_id, target_id, type) IN (
            ($1, $2, 'incoming_request'),
            ($1, $2, 'outgoing_request'),
            ($2, $1, 'incoming_request'),
            ($2, $1, 'outgoing_request')
          )
          RETURNING user_id, target_id, type, nickname, created_at, updated_at
        "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_all(postgres.pool())
      .await?;

      Ok(relationships)
    }

    RelationshipType::Block => {
      let relationships = sqlx::query_as::<_, Relationship>(
        r#"
          DELETE FROM relationships
          WHERE user_id = $1
            AND target_id = $2
            AND type = 'block'
          RETURNING user_id, target_id, type, nickname, created_at, updated_at
        "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_all(postgres.pool())
      .await?;

      Ok(relationships)
    }
  }
}
