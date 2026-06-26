use crate::config::structs::{
  database::DatabaseConfig, features::FeatureConfig, instance::InstanceConfig,
  server::ServerConfig,
};

use serde::Deserialize;
use std::fs;
use std::path::Path;

pub mod errors;
pub mod structs;
pub use errors::ConfigError;

#[derive(Debug, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub is_production: bool,
  pub instance: InstanceConfig,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  #[serde(default)]
  pub features: FeatureConfig,
}

impl Config {
  pub fn load() -> Result<Self, ConfigError> {
    let path = std::env::var("DENDRYTE_CONFIG")
      .unwrap_or_else(|_| "/var/dendryte/conf.toml".into());

    if !Path::new(&path).exists() {
      return Err(ConfigError::NotFound);
    }

    let content = fs::read_to_string(&path).map_err(ConfigError::from)?;
    let config: Config = toml::from_str(&content).map_err(ConfigError::from)?;
    Ok(config)
  }
}
