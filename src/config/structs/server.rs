use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  pub host: String,
  pub port: u16,
  pub cors: CorsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
  pub allowed_origins: Vec<String>,
  pub allow_credentials: bool,
}
