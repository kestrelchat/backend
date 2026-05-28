use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FeatureConfig {
    pub hcaptcha: HCaptcha,
    pub registration: Registration,
}

#[derive(Debug, Deserialize)]
pub struct HCaptcha {
    pub enabled: bool,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct Registration {
    pub enabled: bool,
    pub minimum_age: u32,
}
