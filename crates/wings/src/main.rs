extern crate rocket;
extern crate rocket_okapi;
extern crate serde_json;

pub mod routes;

use kestrel_config::config;
use rocket::{Build, Rocket, launch};
use rocket::fairing::AdHoc;
use std::net::Ipv4Addr;

/// Build and configure the Rocket instance for the web server.
pub async fn web() -> Rocket<Build> {
    let config = config().await;

    // Initialize logging & Sentry from config.sentry.api
    let dsn = if !config.sentry.api.is_empty() {
        config.sentry.api.clone()
    } else {
        std::env::var("SENTRY_DSN").unwrap_or_default()
    };

    let sentry_guard = kestrel_config::setup_logging(
        concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")),
        dsn,
    )
    .await;

    let swagger = rocket_okapi::swagger_ui::make_swagger_ui(
        &rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "/openapi.json".to_owned(),
            ..Default::default()
        },
    )
    .into();

    let mut rocket = rocket::build();

    // If Sentry was initialized, keep the guard alive by managing it in Rocket state
    if let Some(guard) = sentry_guard {
        rocket = rocket.manage(guard);
    }

    // Fairing: report server errors (5xx) to Sentry
    rocket = rocket.attach(AdHoc::on_response("sentry-report", |req, res| {
        Box::pin(async move {
            if res.status().class().is_server_error() {
                kestrel_config::capture_message(&format!("{} {} -> {}", req.method(), req.uri(), res.status()), kestrel_config::Level::Error);
            }
        })
    }));

    // Choose bind address based on environment: localhost for non-production
    let bind_addr = if config.is_production {
        Ipv4Addr::new(0, 0, 0, 0)
    } else {
        Ipv4Addr::new(127, 0, 0, 1)
    };

    // Delegate application route mounting to the routes module, then
    // attach the Swagger UI and configure server address/port.
    routes::mount(config, rocket)
        .mount("/swagger", swagger)
        .configure(rocket::Config {
            address: bind_addr.into(),
            port: 8000,
            ..Default::default()
        })
}

#[launch]
async fn rocket() -> _ {
    web().await
}