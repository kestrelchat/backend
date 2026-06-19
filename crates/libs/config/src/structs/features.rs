use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FeatureConfig {
  pub hcaptcha: HCaptchaConfig,
  pub registration: RegistrationConfig,
}

#[derive(Debug, Deserialize)]
pub struct HCaptchaConfig {
  pub enabled: bool,
  pub sitekey: Option<String>,
  pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationConfig {
  pub enabled: bool,
  pub minimum_age: u32,
}

/// A fixed-duration period in which requests are counted and rate-limited.
#[derive(Debug, Deserialize)]
pub struct RateLimitWindow {
  /// The maximum number of requests in the window.
  pub max: u64,
  /// The duration of the window.
  pub duration: Duration,
}

/// A fixed-capacity container that holds tokens for rate-limiting.
#[derive(Debug, Deserialize)]
pub struct RateLimitBucket {
  /// The capacity of the bucket.
  pub capacity: u64,
  /// The cost of using the bucket.
  pub use_cost: u64,
  /// The fill interval of the bucket.
  pub fill_interval: Duration,
  /// The fill step of the bucket.
  pub fill_step: u64,
}

/// The rate limit configuration for a resource.
#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
  /// The short window.
  pub short_window: RateLimitWindow,
  /// The long window.
  pub long_window: RateLimitWindow,
  /// The bucket.
  pub bucket: RateLimitBucket,
}

/// The rate limit configuration for the entire system.
#[derive(Debug, Deserialize)]
pub struct SystemRateLimitConfig {
  /// The configuration for the "global" resource.
  pub global: RateLimitConfig,
  /// The default configuration for an endpoint.
  pub standard: RateLimitConfig,
  /// The configuration for endpoints with custom rate limiting.
  pub custom: HashMap<String, RateLimitConfig>,
}
