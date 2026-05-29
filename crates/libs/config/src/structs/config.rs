use serde::Deserialize;

use crate::structs::{
  database::DatabaseConfig, features::FeatureConfig, instance::InstanceConfig,
  server::ServerConfig,
};

#[derive(Debug, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub is_production: bool,
  pub instance: InstanceConfig,
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub features: FeatureConfig,
}
