#[derive(Debug)]
pub enum TokenError {
  InvalidLength,
  InvalidEncoding,
  UnsupportedVersion,
  UnknownType,
}
