use std::fmt::Display;

use kestrel_config::structs::features::{
  RateLimitConfig, SystemRateLimitConfig,
};
use redis::{Script, ScriptInvocation};
use rustc_hash::FxHashMap;

use crate::{connection::Redis, error::RedisError};

/// Represents the user ID, either an IP or a user ID.
pub enum RateLimitUserId {
  Ip(String),
  User(String),
}

impl Display for RateLimitUserId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RateLimitUserId::Ip(ip) => write!(f, "ip:{}", ip),
      RateLimitUserId::User(user) => write!(f, "user:{}", user),
    }
  }
}

/// The compiled scripts for rate limiting, mapped by endpoint.
pub struct CompiledRateLimiter {
  /// The script for the global rate limit.
  pub global: Script,
  /// The default script for endpoints without custom configurations.
  pub standard: Script,
  /// Scripts for endpoints with custom configurations.
  ///
  /// [`FxHashMap`] is used, because rate limiting is a performance-critical operation.
  /// The keys are not controlled by users, and therefore cannot be used for HashDoS.
  pub custom: FxHashMap<String, Script>,
}

impl CompiledRateLimiter {
  /// Compiles the rate limit configuration into a Lua table string format.
  fn compile_config(config: &RateLimitConfig) -> String {
    format!(
      "{{ short_window = {{ max = {}, duration = {} }}, long_window = {{ max = {}, duration = {} }}, bucket = {{ capacity = {}, use_cost = {}, fill_interval = {}, fill_step = {} }} }}",
      config.short_window.max,
      config.short_window.duration.as_millis(),
      config.long_window.max,
      config.long_window.duration.as_millis(),
      config.bucket.capacity,
      config.bucket.use_cost,
      config.bucket.fill_interval.as_millis(),
      config.bucket.fill_step
    )
  }
}

/// The script template for the rate limit script.
const SCRIPT_TEMPLATE: &str = include_str!("use_endpoint.lua");

impl From<&'_ SystemRateLimitConfig> for CompiledRateLimiter {
  /// Compiles the rate limit scripts for the given configuration.
  fn from(config: &'_ SystemRateLimitConfig) -> Self {
    let compile_script = |cfg: &RateLimitConfig| {
      let lua_config = format!("local config = {}", Self::compile_config(cfg));
      let script =
        SCRIPT_TEMPLATE.replace("-- CONFIGURATION_PLACEHOLDER", &lua_config);
      Script::new(&script)
    };

    let mut custom = FxHashMap::default();
    custom.reserve(config.custom.len() * 2);
    for (key, val) in &config.custom {
      custom.insert(key.clone(), compile_script(val));
    }

    Self {
      global: compile_script(&config.global),
      standard: compile_script(&config.standard),
      custom,
    }
  }
}

impl CompiledRateLimiter {
  /// Returns the appropriate script for the specified endpoint.
  pub fn get_endpoint_script(&self, endpoint: &str) -> &Script {
    self.custom.get(endpoint).unwrap_or(&self.standard)
  }
}

/// Prepares an invocation of the rate limit script for the given resource and user.
fn prepare_invoke<'sc>(
  script: &'sc Script,
  resource: &str,
  user: &RateLimitUserId,
) -> ScriptInvocation<'sc> {
  let mut invocation = script.prepare_invoke();
  invocation.key(format!("rate-limit:{{{user}:{resource}}}:updated-at"));
  invocation.key(format!("rate-limit:{{{user}:{resource}}}:bucket"));
  invocation.key(format!("rate-limit:{{{user}:{resource}}}:short-window"));
  invocation.key(format!("rate-limit:{{{user}:{resource}}}:long-window"));
  invocation
}

/// Uses the endpoint rate limit for the given user and endpoint, returning an error if the limit is exceeded.
///
/// Returns the delay in seconds that's zero if the limit is not exceeded, or the time to wait if the limit is exceeded.
pub async fn use_endpoint(
  limiter: &CompiledRateLimiter,
  redis: &Redis,
  endpoint: &str,
  user: &RateLimitUserId,
) -> Result<u64, RedisError> {
  let mut conn = redis.conn().clone();

  let global_script = &limiter.global;
  let global_wait: u64 = prepare_invoke(global_script, "global", user)
    .invoke_async(&mut conn)
    .await
    .map_err(RedisError::Redis)?;

  if global_wait > 0 {
    return Ok(global_wait);
  }

  let endpoint_script = limiter.get_endpoint_script(endpoint);
  let endpoint_wait: u64 = prepare_invoke(endpoint_script, endpoint, user)
    .invoke_async(&mut conn)
    .await
    .map_err(RedisError::Redis)?;

  Ok(endpoint_wait)
}
