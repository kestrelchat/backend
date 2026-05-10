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
    TooShort,
    TooLong,
    InvalidCharacters,
    StartsWithInvalidChar,
    EndsWithInvalidChar,
    ConsecutiveSeparators,
}

pub async fn validate(username: &str) -> Result<(), ValidationError> {
    let username = username.trim();

    if username.is_empty() {
        return Err(ValidationError::Empty);
    }

    if username.len() < 2 {
        return Err(ValidationError::TooShort);
    }

    if username.len() > 32 {
        return Err(ValidationError::TooLong);
    }

    let mut last_was_sep = false;

    for (i, c) in username.chars().enumerate() {
        let is_valid = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.';

        if !is_valid {
            return Err(ValidationError::InvalidCharacters);
        }

        let is_sep = c == '_' || c == '.';

        if i == 0 && is_sep {
            return Err(ValidationError::StartsWithInvalidChar);
        }

        if is_sep && last_was_sep {
            return Err(ValidationError::ConsecutiveSeparators);
        }

        last_was_sep = is_sep;
    }

    if let Some(last) = username.chars().last()
        && (last == '_' || last == '.')
    {
        return Err(ValidationError::EndsWithInvalidChar);
    }

    Ok(())
}
