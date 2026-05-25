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

pub mod auth;
pub mod meta;

use rocket::{Build, Rocket};
use rocket_okapi::{
    mount_endpoints_and_merged_docs, openapi_get_routes_spec, settings::OpenApiSettings,
};

pub fn mount(mut rocket: Rocket<Build>) -> Rocket<Build> {
    let settings = OpenApiSettings::default();
    mount_endpoints_and_merged_docs!(
        rocket,
        "/".to_owned(),
        settings,
        "/"    => openapi_get_routes_spec![meta::meta, meta::users_count],
        "/auth" => auth::routes(),
    );
    rocket
}
