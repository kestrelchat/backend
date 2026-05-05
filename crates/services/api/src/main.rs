/*
 * Kestrel - a modern instant-messaging service written in Rust
 * Copyright (C) 2026 Kestrel Chat
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

pub mod routes;
pub mod utils;

use std::net::IpAddr;

use config::Config as AppConfig;
use database::connection::Database;
use rocket::Config as RocketConfig;

use crate::utils::cors::{CorsFairing, preflight};
use utils::errors::{
    bad_request, default_catcher, forbidden, internal_error, method_not_allowed, not_acceptable,
    not_found, service_unavailable, too_many_requests, unauthorized, unprocessable_entity,
};

#[macro_use]
extern crate rocket;
extern crate rocket_okapi;

async fn web() -> Result<rocket::Rocket<rocket::Build>, Box<dyn std::error::Error>> {
    let config = AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;

    let addr: IpAddr = config
        .network
        .host
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("Invalid host address: {}", e))?;

    let rocket_config = RocketConfig {
        address: addr,
        port: config.network.ports.api,
        ..RocketConfig::default()
    };

    let database = Database::connect(&config.database.postgres).await.map_err(
        |e| -> Box<dyn std::error::Error> {
            format!("Failed to connect to database: {}", e).into()
        },
    )?;

    database
        .migrate()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> {
            format!("Failed to run database migrations: {}", e).into()
        })?;

    let swagger =
        rocket_okapi::swagger_ui::make_swagger_ui(&rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "/openapi.json".to_owned(),
            ..Default::default()
        });

    let cors = CorsFairing {
        config: config.network.cors.clone(),
    };

    let rocket = rocket::custom(rocket_config)
        .attach(cors)
        .manage(database)
        .manage(config)
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

#[launch]
async fn rocket() -> _ {
    match web().await {
        Ok(rocket) => rocket,
        Err(e) => {
            eprintln!("Failed to initialize Rocket: {}", e);
            std::process::exit(1);
        }
    }
}
