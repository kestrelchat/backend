use chrono::Utc;
use kestrel_common::{
    models::Session,
    token::{Token, TokenType},
    utils::hasher,
};
use ulid::Ulid;

use crate::{connection::Database, error::DatabaseError};

pub struct PgCreatedSession {
    pub session: Session,
    pub refresh_token: String,
}

pub struct SessionMetadata {
    pub ip_address: Option<std::net::IpAddr>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub user_agent: Option<String>,
    pub operating_system: Option<String>,
    pub platform: Option<String>,
}

pub async fn create_session(
    db: &Database,
    user_id: &str,
    meta: SessionMetadata,
) -> Result<PgCreatedSession, DatabaseError> {
    let id = Ulid::new().to_string();
    let created_at = Utc::now();
    let updated_at = created_at;
    let expires_at = created_at + chrono::Duration::days(30);

    let refresh_token = Token::generate(TokenType::Refresh);

    let refresh_token_hash = hasher::hash(refresh_token.as_bytes());

    let session = sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (
            id,
            user_id,
            refresh_token,
            ip_address,
            country,
            region,
            city,
            user_agent,
            operating_system,
            platform,
            created_at,
            updated_at,
            expires_at,
            last_used_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(&refresh_token_hash)
    .bind(meta.ip_address.map(|ip| ip.to_string()))
    .bind(meta.country)
    .bind(meta.region)
    .bind(meta.city)
    .bind(meta.user_agent)
    .bind(meta.operating_system)
    .bind(meta.platform)
    .bind(created_at)
    .bind(updated_at)
    .bind(expires_at)
    .bind(created_at)
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(PgCreatedSession {
        session,
        refresh_token,
    })
}
