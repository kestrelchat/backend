mod gateway;
mod handlers;
mod protocol;

use std::net::IpAddr;

use kestrel_config::Config as AppConfig;
use rocket::Config as RocketConfig;

use crate::gateway::gateway_route;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
  match gateway() {
    Ok(rocket) => rocket,
    Err(e) => {
      eprintln!("Failed to initialize Rocket: {}", e);
      std::process::exit(1);
    }
  }
}

fn gateway() -> Result<rocket::Rocket<rocket::Build>, Box<dyn std::error::Error>>
{
  let config =
    AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;

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
    port: config.server.ports.gateway,
    ..RocketConfig::default()
  };

  let rocket = rocket::custom(rocket_config)
    .manage(config)
    .mount("/", routes![gateway_route]);

  Ok(rocket)
}
