use sqlx::PgPool;

use crate::{
  authorization::model::guild::GuildPermission,
  database::postgres::error::DatabaseError,
};

pub struct PermissionResolver {
  pool: PgPool,
}

impl PermissionResolver {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn compute_guild(
    &self,
    user_id: &str,
    guild_id: &str,
  ) -> Result<GuildPermission, DatabaseError> {
    let is_member: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (SELECT 1 FROM guild_members WHERE user_id = $1 AND guild_id = $2)
            "#
        )
        .bind(user_id)
        .bind(guild_id)
        .fetch_one(&self.pool)
        .await?;

    if !is_member {
      return Err(DatabaseError::NotFound);
    }

    let bits: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(BIT_OR(gr.permissions), 0)
      FROM guild_roles gr
      JOIN member_roles mr ON mr.role_id = gr.id
      WHERE mr.user_id = $1 AND mr.guild_id = $2
      "#,
    )
    .bind(user_id)
    .bind(guild_id)
    .fetch_one(&self.pool)
    .await?;

    Ok(GuildPermission::from_bits(bits as u64))
  }
}
