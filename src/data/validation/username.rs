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
    let is_valid =
      c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.';

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
