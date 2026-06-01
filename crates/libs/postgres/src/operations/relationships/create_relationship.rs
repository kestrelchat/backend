use chrono::Utc;

use crate::{connection::Database, error::DatabaseError};
use kestrel_common::models::{Relationship, relationship::RelationshipAction};

pub async fn create_relationship(
  db: &Database,
  user_id: &str,
  target_id: &str,
  relationship_action: RelationshipAction,
) -> Result<Vec<Relationship>, DatabaseError> {
  let created_at = Utc::now();
  let updated_at = created_at;

  match relationship_action {
    RelationshipAction::Friend => {
      let either_blocked: bool = sqlx::query_scalar(
        r#"
                SELECT EXISTS (
                    SELECT 1 FROM relationships
                    WHERE (
                        (user_id = $1 AND target_id = $2)
                     OR (user_id = $2 AND target_id = $1)
                    )
                    AND type = 'block'
                )
                "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_one(db.pool())
      .await?;

      if either_blocked {
        return Err(DatabaseError::InvalidOperation(
          "RELATIONSHIP_FAILED".to_string(),
        ));
      }

      let pending: Option<(String, String)> = sqlx::query_as(
        r#"
                SELECT user_id, target_id
                FROM relationships
                WHERE (
                    (user_id = $1 AND target_id = $2)
                 OR (user_id = $2 AND target_id = $1)
                )
                AND type IN ('incoming_request', 'outgoing_request')
                LIMIT 1
                "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_optional(db.pool())
      .await?;

      if let Some((requester_id, _)) = pending {
        if requester_id == target_id {
          sqlx::query(
            r#"
                        UPDATE relationships
                        SET type = 'friend', updated_at = $3
                        WHERE (
                            (user_id = $1 AND target_id = $2)
                         OR (user_id = $2 AND target_id = $1)
                        )
                        AND type IN ('incoming_request', 'outgoing_request')
                        "#,
          )
          .bind(user_id)
          .bind(target_id)
          .bind(updated_at)
          .execute(db.pool())
          .await?;

          let relationships: Vec<Relationship> = sqlx::query_as(
                        r#"
                        SELECT user_id, target_id, type, nickname, created_at, updated_at
                        FROM relationships
                        WHERE (
                            (user_id = $1 AND target_id = $2)
                         OR (user_id = $2 AND target_id = $1)
                        )
                        AND type = 'friend'
                        "#,
                    )
                    .bind(user_id)
                    .bind(target_id)
                    .fetch_all(db.pool())
                    .await?;

          return Ok(relationships);
        } else {
          return Err(DatabaseError::InvalidOperation(
            "REQUEST_ALREADY_SENT".to_string(),
          ));
        }
      }

      let already_friends: bool = sqlx::query_scalar(
        r#"
                SELECT EXISTS (
                    SELECT 1 FROM relationships
                    WHERE user_id = $1
                      AND target_id = $2
                      AND type = 'friend'
                )
                "#,
      )
      .bind(user_id)
      .bind(target_id)
      .fetch_one(db.pool())
      .await?;

      if already_friends {
        return Err(DatabaseError::InvalidOperation(
          "ALREADY_FRIENDS".to_string(),
        ));
      }

      let outgoing = sqlx::query_as::<_, Relationship>(
                r#"
                INSERT INTO relationships (user_id, target_id, type, created_at, updated_at)
                VALUES ($1, $2, 'outgoing_request', $3, $4)
                RETURNING user_id, target_id, type, nickname, created_at, updated_at
                "#,
            )
            .bind(user_id)
            .bind(target_id)
            .bind(updated_at)
            .bind(created_at)
            .fetch_one(db.pool())
            .await?;

      let incoming = sqlx::query_as::<_, Relationship>(
                r#"
                INSERT INTO relationships (user_id, target_id, type, created_at, updated_at)
                VALUES ($1, $2, 'incoming_request', $3, $4)
                RETURNING user_id, target_id, type, nickname, created_at, updated_at
                "#,
            )
            .bind(target_id)
            .bind(user_id)
            .bind(created_at)
            .bind(updated_at)
            .fetch_one(db.pool())
            .await?;

      Ok(vec![outgoing, incoming])
    }

    RelationshipAction::Block => {
      // remove any existing friendship
      sqlx::query(
        r#"
                DELETE FROM relationships
                WHERE (
                    (user_id = $1 AND target_id = $2)
                 OR (user_id = $2 AND target_id = $1)
                )
                AND type IN ('friend', 'incoming_request', 'outgoing_request')
                "#,
      )
      .bind(user_id)
      .bind(target_id)
      .execute(db.pool())
      .await?;

      let blocked = sqlx::query_as::<_, Relationship>(
                r#"
                INSERT INTO relationships (user_id, target_id, type, created_at, updated_at)
                VALUES ($1, $2, 'block', $3, $4)
                RETURNING user_id, target_id, type, nickname, created_at, updated_at
                "#,
            )
            .bind(user_id)
            .bind(target_id)
            .bind(updated_at)
            .bind(created_at)
            .fetch_one(db.pool())
            .await?;

      Ok(vec![blocked])
    }
  }
}
