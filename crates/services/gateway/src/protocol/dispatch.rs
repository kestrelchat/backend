pub enum DispatchEvent {
  Test,
  Unknown(String),
}

impl DispatchEvent {
  pub fn from_str(s: &str) -> Self {
    match s {
      "dispatch.test" => Self::Test,
      other => Self::Unknown(other.into()),
    }
  }
}
