#[derive(Debug)]
pub enum ValidationError {
  Empty,
  TooLong,
  MissingAt,
  InvalidStructure,
  InvalidDomain,
}

pub async fn validate(
  email: &str,
  production: bool,
) -> Result<(), ValidationError> {
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

  let invalid = (!domain.contains('.') && domain != "localhost")
    || (domain == "localhost" && production);

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
