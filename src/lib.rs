pub mod catchers;
pub mod guards;
pub mod routes;
pub mod utils;

use std::net::IpAddr;

use kestrel_common::utils::geoip::GeoIpClient;
use kestrel_config::Config as AppConfig;
use kestrel_postgres::connection::Database;
use kestrel_redis::{
  connection::Redis,
  operations::rate_limiting::use_endpoint::CompiledRateLimiter,
};
use rocket::Config as RocketConfig;

use crate::{
  catchers::too_many_requests::too_many_requests,
  utils::cors::{CorsFairing, preflight},
};
use utils::errors::{
  bad_request, default_catcher, forbidden, internal_error, method_not_allowed,
  not_acceptable, not_found, service_unavailable, unauthorized,
  unprocessable_entity,
};

#[macro_use]
extern crate rocket;
extern crate rocket_okapi;

pub async fn web(
  config_override: Option<AppConfig>,
) -> Result<rocket::Rocket<rocket::Build>, Box<dyn std::error::Error>> {
  let config = if let Some(config_override) = config_override {
    config_override
  } else {
    AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?
  };

  let addr: IpAddr =
    config
      .server
      .host
      .parse()
      .map_err(|e: std::net::AddrParseError| {
        format!("Invalid host address: {}", e)
      })?;

  let rocket_config = RocketConfig {
    address: addr,
    port: config.server.port,
    ..RocketConfig::default()
  };

  let postgres = Database::connect(&config.database.postgres.url)
    .await
    .map_err(|e| -> Box<dyn std::error::Error> {
      format!("Failed to connect to postgres: {}", e).into()
    })?;

  let redis = Redis::connect(&config.database.redis.url).await.map_err(
    |e| -> Box<dyn std::error::Error> {
      format!("Failed to connect to redis: {}", e).into()
    },
  )?;

  postgres
    .migrate()
    .await
    .map_err(|e| -> Box<dyn std::error::Error> {
      format!("Failed to run database migrations: {}", e).into()
    })?;

  let swagger = rocket_okapi::swagger_ui::make_swagger_ui(
    &rocket_okapi::swagger_ui::SwaggerUIConfig {
      url: "/openapi.json".to_owned(),
      ..Default::default()
    },
  );

  let cors = CorsFairing {
    config: config.server.cors.clone(),
  };

  let geoip = GeoIpClient::default();

  let rate_limiter = CompiledRateLimiter::from(&config.features.rate_limiting);
  rate_limiter.warm_up(&redis).await.ok();

  let rocket = rocket::custom(rocket_config)
    .attach(cors)
    .manage(postgres)
    .manage(redis)
    .manage(config)
    .manage(rate_limiter)
    .manage(geoip)
    .mount("/", routes![preflight])
    .mount("/swagger", swagger)
    .register(
      "/",
      catchers![
        bad_request,
        unauthorized,
        forbidden,
        not_found,
        method_not_allowed,
        not_acceptable,
        unprocessable_entity,
        too_many_requests,
        internal_error,
        service_unavailable,
        default_catcher,
      ],
    );

  Ok(routes::mount(rocket))
}
