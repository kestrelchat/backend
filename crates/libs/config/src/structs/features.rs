use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct FeatureConfig {
  #[serde(default)]
  pub hcaptcha: HCaptchaConfig,
  #[serde(default)]
  pub registration: RegistrationConfig,
  #[serde(default)]
  pub rate_limiting: SystemRateLimitConfig,
}

#[derive(Debug, Deserialize)]
pub struct HCaptchaConfig {
  pub enabled: bool,
  pub sitekey: Option<String>,
  pub secret: Option<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for HCaptchaConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      sitekey: None,
      secret: None,
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct RegistrationConfig {
  pub enabled: bool,
  pub minimum_age: u32,
}

impl Default for RegistrationConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      minimum_age: 0,
    }
  }
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
  pub capacity: f64,
  /// The cost of using the bucket.
  pub use_cost: f64,
  /// The duration for which one token is added.
  pub duration: Duration,
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

impl RateLimitConfig {
  pub fn global_default() -> Self {
    RateLimitConfig {
      short_window: RateLimitWindow {
        max: 50,
        duration: Duration::from_secs(2),
      },
      long_window: RateLimitWindow {
        max: 500,
        duration: Duration::from_secs(60),
      },
      bucket: RateLimitBucket {
        capacity: 500.0,
        use_cost: 1.0,
        duration: Duration::from_millis(500),
      },
    }
  }

  pub fn per_endpoint_default() -> Self {
    RateLimitConfig {
      short_window: RateLimitWindow {
        max: 10,
        duration: Duration::from_secs(1),
      },
      long_window: RateLimitWindow {
        max: 60,
        duration: Duration::from_secs(60),
      },
      bucket: RateLimitBucket {
        capacity: 60.0,
        use_cost: 1.0,
        duration: Duration::from_secs(1),
      },
    }
  }
}

/// The rate limit configuration for the entire system.
#[derive(Debug, Deserialize)]
pub struct SystemRateLimitConfig {
  /// The configuration for the "global" resource.
  #[serde(default = "RateLimitConfig::global_default")]
  pub global: RateLimitConfig,
  /// The default configuration for an endpoint.
  #[serde(default = "RateLimitConfig::per_endpoint_default")]
  pub standard: RateLimitConfig,
  /// The configuration for endpoints with custom rate limiting.
  #[serde(default)]
  pub custom: HashMap<String, RateLimitConfig>,
}

impl Default for SystemRateLimitConfig {
  fn default() -> Self {
    Self {
      global: RateLimitConfig::global_default(),
      standard: RateLimitConfig::per_endpoint_default(),
      custom: HashMap::new(),
    }
  }
}
