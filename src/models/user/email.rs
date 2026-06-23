#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl AsRef<str> for Email {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<Email> for String {
  fn from(addr: Email) -> Self {
    addr.0
  }
}

#[derive(Debug)]
pub enum ValidationError {
  Empty,
  TooLong,
  MissingAt,
  InvalidStructure,
  InvalidDomain,
}

impl Email {
  /// Creates a new `EmailAddress` while normalizing the email's structure.
  pub fn new(
    email: &str,
    is_production: bool,
  ) -> Result<Self, ValidationError> {
    let email = email.to_lowercase();

    if email.is_empty() || email.chars().all(|c| c.is_whitespace()) {
      return Err(ValidationError::Empty);
    }

    // This check is simplified wrt the full RFC
    let (local, domain) =
      email.rsplit_once('@').ok_or(ValidationError::MissingAt)?;

    if domain.contains('@') {
      return Err(ValidationError::InvalidStructure);
    }

    if local.len() > 64 || domain.len() > 255 {
      return Err(ValidationError::TooLong);
    }

    if local.is_empty() || domain.is_empty() {
      return Err(ValidationError::InvalidStructure);
    }

    let invalid = (!domain.contains('.') && domain != "localhost")
      || (domain == "localhost" && is_production);

    if invalid {
      return Err(ValidationError::InvalidDomain);
    }

    if domain.starts_with('.') || domain.ends_with('.') {
      return Err(ValidationError::InvalidDomain);
    }

    if domain.contains("..") {
      return Err(ValidationError::InvalidDomain);
    }

    Ok(Self(email))
  }

  /// Creates a new `EmailAddress` without checking the email's structure.
  ///
  /// Use this when the email is already known to be valid, for example when
  /// reading from the database.
  pub fn new_unchecked(email: String) -> Self {
    Self(email)
  }
}
