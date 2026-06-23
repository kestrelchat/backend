use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
  NotFound,
  EnvVar(String),
  ParseError(toml::de::Error),
  Io(std::io::Error),
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConfigError::NotFound => {
        write!(f, "Config file not found")
      }
      ConfigError::EnvVar(e) => {
        write!(f, "Environment variable error: {}", e)
      }
      ConfigError::ParseError(e) => {
        write!(f, "Failed to parse config: {}", e)
      }
      ConfigError::Io(e) => {
        write!(f, "IO error: {}", e)
      }
    }
  }
}

impl Error for ConfigError {}

impl From<toml::de::Error> for ConfigError {
  fn from(err: toml::de::Error) -> Self {
    ConfigError::ParseError(err)
  }
}

impl From<std::io::Error> for ConfigError {
  fn from(err: std::io::Error) -> Self {
    ConfigError::Io(err)
  }
}
