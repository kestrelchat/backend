use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FeatureConfig {
  pub hcaptcha: HCaptchaConfig,
  pub registration: RegistrationConfig,
}

#[derive(Debug, Deserialize)]
pub struct HCaptchaConfig {
  pub enabled: bool,
  pub sitekey: Option<String>,
  pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationConfig {
  pub enabled: bool,
  pub minimum_age: u32,
}
