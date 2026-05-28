use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub ports: PortsConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Deserialize)]
pub struct PortsConfig {
    pub gateway: u16,
    pub api: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
}
