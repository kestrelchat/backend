use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InstanceConfig {
  pub name: String,
  pub domain: String,
  pub description: Option<String>,
}
