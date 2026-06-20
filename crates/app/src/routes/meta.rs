use kestrel_config::Config;
use rocket::serde::json::Json;
use rocket_okapi::okapi::schemars;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Meta {
  pub kestrel: String,
  pub instance: InstanceMeta,
  pub features: FeaturesMeta,
}

#[derive(Serialize, JsonSchema)]
pub struct InstanceMeta {
  pub name: String,
  pub domain: String,
  pub description: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct FeaturesMeta {
  pub hcaptcha: HCaptchaMeta,
  pub registration: RegistrationMeta,
}

#[derive(Serialize, JsonSchema)]
pub struct HCaptchaMeta {
  pub enabled: bool,
  pub sitekey: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct RegistrationMeta {
  pub enabled: bool,
  pub minimum_age: u32,
}

#[openapi(tag = "Core")]
#[get("/")]
pub fn meta(config: &rocket::State<Config>) -> Json<Meta> {
  Json(Meta {
    kestrel: env!("CARGO_PKG_VERSION").into(),
    instance: InstanceMeta {
      name: config.instance.name.clone(),
      domain: config.instance.domain.clone(),
      description: config.instance.description.clone(),
    },
    features: FeaturesMeta {
      hcaptcha: HCaptchaMeta {
        enabled: config.features.hcaptcha.enabled,
        sitekey: config.features.hcaptcha.sitekey.clone(),
      },
      registration: RegistrationMeta {
        enabled: config.features.registration.enabled,
        minimum_age: config.features.registration.minimum_age,
      },
    },
  })
}
