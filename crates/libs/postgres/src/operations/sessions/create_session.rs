use chrono::Utc;
use kestrel_common::{models::Session, token::Token, utils::hasher};
use ulid::Ulid;

use crate::{connection::Database, error::DatabaseError};

pub struct CreatedSession {
    pub session: Session,
    pub refresh_token: String,
}

pub async fn create_session(
    db: &Database,
    user_id: &str,
    ip_address: Option<std::net::IpAddr>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    user_agent: Option<String>,
    operating_system: Option<String>,
    platform: Option<String>,
) -> Result<CreatedSession, DatabaseError> {
    let id = Ulid::new().to_string();
    let created_at = Utc::now();
    let updated_at = created_at;
    let expires_at = created_at + chrono::Duration::days(30);

    let refresh_token = Token::generate(1);

    let refresh_token_hash = hasher::hash(refresh_token.as_bytes())
        .await
        .map_err(|_| DatabaseError::Other("failed to hash refresh token".to_string()))?;

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
            last_used_at,
            revoked_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(&refresh_token_hash)
    .bind(ip_address.map(|ip| ip.to_string()))
    .bind(country)
    .bind(region)
    .bind(city)
    .bind(user_agent)
    .bind(operating_system)
    .bind(platform)
    .bind(created_at)
    .bind(updated_at)
    .bind(expires_at)
    .bind(created_at)
    .bind(None::<chrono::DateTime<Utc>>)
    .fetch_one(db.pool())
    .await
    .map_err(DatabaseError::from_sqlx)?;

    Ok(CreatedSession {
        session,
        refresh_token,
    })
}
