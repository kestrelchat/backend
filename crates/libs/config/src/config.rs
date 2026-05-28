use crate::errors::ConfigError;
use crate::structs::Config;
use std::fs;
use std::path::Path;

impl Config {
  pub fn load() -> Result<Self, ConfigError> {
    let path = std::env::var("KESTREL_CONFIG")
      .unwrap_or_else(|_| "/var/kestrel/conf.toml".into());

    if !Path::new(&path).exists() {
      return Err(ConfigError::NotFound);
    }

    let content = fs::read_to_string(&path).map_err(ConfigError::from)?;
    let config: Config = toml::from_str(&content).map_err(ConfigError::from)?;
    Ok(config)
  }
}
