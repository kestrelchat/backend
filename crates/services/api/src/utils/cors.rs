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

use kestrel_config::structs::network::Cors;
use rocket::{
    Request, Response,
    fairing::{Fairing, Info, Kind},
    http::{Header, Method, Status},
    response::status::NoContent,
};

#[options("/<_..>")]
pub fn preflight() -> NoContent {
    NoContent
}

pub struct CorsFairing {
    pub config: Cors,
}

#[rocket::async_trait]
impl Fairing for CorsFairing {
    fn info(&self) -> Info {
        Info {
            name: "Kestrel REST API CORS Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let origin = req.headers().get_one("Origin");

        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ));

        res.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));

        if req.method() == Method::Options {
            res.set_header(Header::new("Access-Control-Max-Age", "86400"));
            res.set_status(Status::NoContent);
        }

        let allow_all = self.config.allowed_origins.contains(&"*".into());

        if self.config.allow_credentials {
            if let Some(origin) = origin
                && (allow_all || self.config.allowed_origins.contains(&origin.into()))
            {
                res.set_header(Header::new("Access-Control-Allow-Origin", origin));
                res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
                res.set_header(Header::new("Vary", "Origin"));
            }
        } else {
            if allow_all {
                res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
            } else if let Some(origin) = origin
                && self.config.allowed_origins.contains(&origin.into())
            {
                res.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        }
    }
}
