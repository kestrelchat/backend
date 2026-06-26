use crate::{
  api::guards::auth_context::AuthContext,
  database::postgres::{
    connection::Database,
    operations::{
      relationships::get_relationships::get_relationships as postgres_get_relationships,
      user::batch_get_user_summaries as postgres_batch_get_user_summaries,
    },
  },
  errors::AppError,
};
use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RelationshipUser {
  pub id: String,
  pub username: String,
  pub nickname: Option<String>,
  pub discrim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PendingRelationships {
  pub incoming: Vec<RelationshipUser>,
  pub outgoing: Vec<RelationshipUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetRelationshipResponse {
  pub friends: Vec<RelationshipUser>,
  pub pending: PendingRelationships,
  pub blocked: Vec<RelationshipUser>,
}

#[openapi(tag = "Core")]
#[get("/@me/relationships")]
pub async fn get_relationships(
  postgres: &State<Database>,
  auth_ctx: AuthContext,
) -> Result<Json<GetRelationshipResponse>, AppError> {
  let user_id = auth_ctx.user_id;

  let relationships =
    postgres_get_relationships(postgres.inner(), user_id.as_str())
      .await
      .map_err(AppError::from)?;

  let ids: HashSet<String> = relationships
    .friends
    .iter()
    .map(|r| r.target_id.clone())
    .chain(relationships.incoming.iter().map(|r| r.user_id.clone()))
    .chain(relationships.outgoing.iter().map(|r| r.target_id.clone()))
    .chain(relationships.blocked.iter().map(|r| r.target_id.clone()))
    .collect();

  let users = postgres_batch_get_user_summaries(
    postgres.inner(),
    &ids.into_iter().collect::<Vec<_>>(),
  )
  .await
  .map_err(AppError::from)?;

  let user_map: HashMap<String, _> =
    users.into_iter().map(|u| (u.id.clone(), u)).collect();

  Ok(Json(GetRelationshipResponse {
    friends: relationships
      .friends
      .into_iter()
      .filter_map(|r| {
        user_map.get(&r.target_id).map(|u| RelationshipUser {
          id: u.id.clone(),
          username: u.username.clone(),
          nickname: r.nickname,
          discrim: u.discrim.clone(),
        })
      })
      .collect(),

    pending: PendingRelationships {
      incoming: relationships
        .incoming
        .into_iter()
        .filter_map(|r| {
          user_map.get(&r.user_id).map(|u| RelationshipUser {
            id: u.id.clone(),
            username: u.username.clone(),
            nickname: r.nickname,
            discrim: u.discrim.clone(),
          })
        })
        .collect(),

      outgoing: relationships
        .outgoing
        .into_iter()
        .filter_map(|r| {
          user_map.get(&r.target_id).map(|u| RelationshipUser {
            id: u.id.clone(),
            username: u.username.clone(),
            nickname: r.nickname,
            discrim: u.discrim.clone(),
          })
        })
        .collect(),
    },

    blocked: relationships
      .blocked
      .into_iter()
      .filter_map(|r| {
        user_map.get(&r.target_id).map(|u| RelationshipUser {
          id: u.id.clone(),
          username: u.username.clone(),
          nickname: r.nickname,
          discrim: u.discrim.clone(),
        })
      })
      .collect(),
  }))
}
