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

use crate::errors::ConfigError;
use crate::structs::Config;
use std::fs;
use std::path::Path;

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = std::env::var("KESTREL_CONFIG")
            .unwrap_or_else(|_| "/var/kestrel/conf.toml".to_string());

        if !Path::new(&path).exists() {
            return Err(ConfigError::NotFound);
        }

        let content = fs::read_to_string(&path).map_err(ConfigError::from)?;
        let config: Config = toml::from_str(&content).map_err(ConfigError::from)?;
        Ok(config)
    }
}
