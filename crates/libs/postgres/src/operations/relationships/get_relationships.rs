use kestrel_common::models::{Relationship, RelationshipType};

use crate::{connection::Database, error::DatabaseError};

pub struct RelationshipsBundle {
  pub friends: Vec<Relationship>,
  pub incoming: Vec<Relationship>,
  pub outgoing: Vec<Relationship>,
  pub blocked: Vec<Relationship>,
}

pub async fn get_relationships(
  db: &Database,
  user_id: &str,
) -> Result<RelationshipsBundle, DatabaseError> {
  let rows: Vec<Relationship> = sqlx::query_as::<_, Relationship>(
    r#"
        SELECT user_id, target_id, type, nickname, created_at, updated_at
        FROM relationships
        WHERE user_id = $1 OR target_id = $1
        "#,
  )
  .bind(user_id)
  .fetch_all(db.pool())
  .await?;

  let mut friends = Vec::new();
  let mut incoming = Vec::new();
  let mut outgoing = Vec::new();
  let mut blocked = Vec::new();

  for r in rows {
    match r.relationship_type {
      RelationshipType::Friend => {
        friends.push(r);
      }

      RelationshipType::Block => {
        blocked.push(r);
      }

      RelationshipType::IncomingRequest => {
        incoming.push(r);
      }

      RelationshipType::OutgoingRequest => {
        outgoing.push(r);
      }
    }
  }

  Ok(RelationshipsBundle {
    friends,
    incoming,
    outgoing,
    blocked,
  })
}
