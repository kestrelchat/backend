#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password<'p>(&'p str);

impl<'p> AsRef<str> for Password<'p> {
  fn as_ref(&self) -> &str {
    self.0
  }
}

impl<'p> Password<'p> {
  pub fn new(password: &'p str) -> Result<Self, ValidationError> {
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

    Ok(Self(password))
  }

  pub fn new_unchecked(password: &'p str) -> Self {
    Self(password)
  }
}

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
