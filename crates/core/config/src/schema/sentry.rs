use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Sentry {
	pub api: String,
}

impl Default for Sentry {
	fn default() -> Self {
		Self { api: String::new() }
	}
}

