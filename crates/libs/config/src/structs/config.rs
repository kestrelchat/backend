use serde::Deserialize;

use crate::structs::{
  database::DatabaseConfig, features::FeatureConfig, server::ServerConfig,
};

#[derive(Debug, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub is_production: bool,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub features: FeatureConfig,
}
