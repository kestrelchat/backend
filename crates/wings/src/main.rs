extern crate rocket;
extern crate rocket_okapi;
extern crate serde_json;

pub mod routes;
pub mod errors;

use kestrel_config::config;
use rocket::fairing::AdHoc;
use rocket::{Build, Rocket, launch};
use sentry;
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
        dsn.clone(),
    )
    .await;

    let sentry_guard = if sentry_guard.is_some() {
        sentry_guard
    } else if !dsn.is_empty() {
        let guard = sentry::init(sentry::ClientOptions {
            dsn: dsn.parse().ok(),
            release: Some(concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")).into()),
            environment: Some(
                if config.is_production {
                    "production"
                } else {
                    "development"
                }
                .into(),
            ),
            ..Default::default()
        });
        // Register a panic hook that captures panics and forwards them to Sentry.
        std::panic::set_hook(Box::new(|info| {
            let payload = info.payload();
            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.as_str()
            } else {
                "panic"
            };

            let location = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_default();

            let full = if location.is_empty() {
                format!("panic: {}", msg)
            } else {
                format!("panic at {}: {}", location, msg)
            };

            kestrel_config::capture_message(&full, kestrel_config::Level::Fatal);
        }));
        Some(guard)
    } else {
        None
    };

    let swagger =
        rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "/openapi.json".to_owned(),
            ..Default::default()
        })
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
                kestrel_config::capture_message(
                    &format!("{} {} -> {}", req.method(), req.uri(), res.status()),
                    kestrel_config::Level::Error,
                );
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
    // attach the Swagger UI, register JSON error catchers and configure server address/port.
    routes::mount(config, rocket)
        .mount("/swagger", swagger)
        .register(
            "/",
            rocket::catchers![
                errors::default_catcher,
                errors::bad_request,
                errors::unauthorized,
                errors::forbidden,
                errors::not_found,
                errors::internal_error,
            ],
        )
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
