use serde::Deserialize;

use super::sentry::Sentry;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub is_production: bool,
    #[serde(default)]
    pub sentry: Sentry,
}
