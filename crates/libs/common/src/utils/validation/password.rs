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
  MissingUpper,
  MissingLower,
  MissingDigit,
  MissingSpecial,
}

pub async fn validate(password: &str) -> Result<(), ValidationError> {
  if password.is_empty() || password.chars().all(|c| c.is_whitespace()) {
    return Err(ValidationError::Empty);
  }

  if password.len() < 8 {
    return Err(ValidationError::TooShort);
  }

  if password.len() > 64 {
    return Err(ValidationError::TooLong);
  }

  if !password.chars().any(|c| c.is_uppercase()) {
    return Err(ValidationError::MissingUpper);
  }

  if !password.chars().any(|c| c.is_lowercase()) {
    return Err(ValidationError::MissingLower);
  }

  if !password.chars().any(|c| c.is_ascii_digit()) {
    return Err(ValidationError::MissingDigit);
  }

  if !password
    .chars()
    .any(|c| c.is_ascii_graphic() && !c.is_alphanumeric())
  {
    return Err(ValidationError::MissingSpecial);
  }

  Ok(())
}
