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

mod gateway;
mod handlers;
mod protocol;

use std::net::IpAddr;

use config::Config as AppConfig;
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

fn gateway() -> Result<rocket::Rocket<rocket::Build>, Box<dyn std::error::Error>> {
    let config = AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;

    let addr: IpAddr = config
        .network
        .host
        .parse()
        .map_err(|e: std::net::AddrParseError| format!("Invalid host address: {}", e))?;

    let rocket_config = RocketConfig {
        address: addr,
        port: config.network.ports.gateway,
        ..RocketConfig::default()
    };

    let rocket = rocket::custom(rocket_config)
        .manage(config)
        .mount("/", routes![gateway_route]);

    Ok(rocket)
}
