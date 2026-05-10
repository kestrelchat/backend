// Kestrel - a modern instant-messaging service written in Rust
// Copyright (C) 2026 Kestrel Chat
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct GeoIpResponse {
    pub status: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,

    pub isp: Option<String>,
    pub org: Option<String>,

    pub proxy: Option<bool>,
    pub hosting: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct GeoInfo {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
}

#[derive(Clone, Default)]
pub struct GeoIpClient {
    http: reqwest::Client,
}

impl GeoIpClient {
    pub async fn lookup(&self, ip: IpAddr) -> Option<GeoInfo> {
        let url = format!("{}/json/{}", "http://ip-api.com", ip);

        let resp = self.http.get(&url).send().await.ok()?;

        let bytes = resp.bytes().await.ok()?;
        let data: GeoIpResponse = serde_json::from_slice(&bytes).ok()?;

        if let Some(status) = &data.status
            && status != "success"
        {
            return None;
        }

        Some(GeoInfo {
            country: data.country,
            region: data.region,
            city: data.city,
        })
    }
}
