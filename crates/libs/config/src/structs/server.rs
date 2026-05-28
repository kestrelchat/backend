use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub ports: Ports,
    pub cors: Cors,
}

#[derive(Debug, Deserialize)]
pub struct Ports {
    pub gateway: u16,
    pub api: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cors {
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
}
