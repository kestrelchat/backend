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

use rocket::{
    Request,
    request::{FromRequest, Outcome},
};
use rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

impl<'r> OpenApiFromRequest<'r> for RequestContext {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestContext {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self {
            ip: extract_ip(req),
            user_agent: req.headers().get_one("User-Agent").map(str::to_owned),
        })
    }
}

fn extract_ip(req: &Request<'_>) -> Option<IpAddr> {
    if let Some(ip) = req.headers().get_one("CF-Connecting-IP")
        && let Ok(ip) = ip.parse()
    {
        return Some(ip);
    }

    if let Some(forwarded) = req.headers().get_one("X-Forwarded-For")
        && let Some(ip) = forwarded.split(',').next()
        && let Ok(ip) = ip.trim().parse()
    {
        return Some(ip);
    }

    if let Some(ip) = req.headers().get_one("X-Real-IP")
        && let Ok(ip) = ip.parse()
    {
        return Some(ip);
    }

    req.client_ip()
}
