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

#[derive(Debug)]
pub enum ValidationError {
    Empty,
    TooLong,
    MissingAt,
    InvalidStructure,
    InvalidDomain,
}

pub async fn validate(email: &str, production: bool) -> Result<(), ValidationError> {
    if email.is_empty() || email.chars().all(|c| c.is_whitespace()) {
        return Err(ValidationError::Empty);
    }

    if email.len() > 320 {
        return Err(ValidationError::TooLong);
    }

    let parts: Vec<&str> = email.split('@').collect();

    if parts.len() != 2 {
        return Err(ValidationError::MissingAt);
    }

    let (local, domain) = (parts[0], parts[1]);

    if local.is_empty() || domain.is_empty() {
        return Err(ValidationError::InvalidStructure);
    }

    let invalid =
        (!domain.contains('.') && domain != "localhost") || (domain == "localhost" && production);

    if invalid {
        return Err(ValidationError::InvalidDomain);
    }

    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(ValidationError::InvalidDomain);
    }

    if domain.contains("..") {
        return Err(ValidationError::InvalidDomain);
    }

    Ok(())
}
