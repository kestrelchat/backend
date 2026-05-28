use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub postgres: Postgres,
    pub redis: Redis,
}

#[derive(Debug, Deserialize)]
pub struct Postgres {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    pub url: String,
}
