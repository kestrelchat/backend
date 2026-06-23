use std::{fmt::Display, net::IpAddr, time::SystemTime};

use crate::config::structs::features::{
  RateLimitConfig, SystemRateLimitConfig,
};
use redis::{Script, ScriptInvocation};
use rustc_hash::FxHashMap;

use crate::database::redis::{connection::Redis, error::RedisError};

/// Represents the user ID, either an IP or a user ID.
pub enum RateLimitUserId<'req> {
  Ip(IpAddr),
  User(&'req str),
}

impl Display for RateLimitUserId<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RateLimitUserId::Ip(ip) => write!(f, "ip:{}", ip.to_canonical()),
      RateLimitUserId::User(user) => write!(f, "user:{}", user),
    }
  }
}

/// The compiled scripts for rate limiting, mapped by endpoint.
pub struct CompiledRateLimiter {
  /// The script for the global rate limit.
  global: Script,
  /// The default script for endpoints without custom configurations.
  standard: Script,
  /// Scripts for endpoints with custom configurations.
  ///
  /// [`FxHashMap`] is used, because rate limiting is a performance-critical operation.
  /// The keys are not controlled by users, and therefore cannot be used for HashDoS.
  custom: FxHashMap<String, Script>,
}

impl CompiledRateLimiter {
  /// Uses the endpoint rate limit for the given user and endpoint, returning an error if the limit is exceeded.
  ///
  /// Returns the delay in seconds that's zero if the limit is not exceeded, or the time to wait if the limit is exceeded.
  pub async fn use_endpoint(
    &self,
    redis: &Redis,
    endpoint: &str,
    user: &RateLimitUserId<'_>,
  ) -> Result<u64, RedisError> {
    let mut conn = redis.conn().clone();
    let now_ms = SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_nanos() as f64
      * 1e-6;

    let global_script = &self.global;
    let endpoint_script = self.get_endpoint_script(endpoint);

    let global_invocation =
      Self::prepare_invoke(global_script, "global", user, now_ms);
    let endpoint_invocation =
      Self::prepare_invoke(endpoint_script, endpoint, user, now_ms);

    let global_wait: u64 = global_invocation
      .invoke_async(&mut conn)
      .await
      .map_err(RedisError::Redis)?;
    let endpoint_wait: u64 = endpoint_invocation
      .invoke_async(&mut conn)
      .await
      .map_err(RedisError::Redis)?;

    Ok(global_wait.max(endpoint_wait))
  }

  /// Returns the appropriate script for the specified endpoint.
  fn get_endpoint_script(&self, endpoint: &str) -> &Script {
    self.custom.get(endpoint).unwrap_or(&self.standard)
  }

  /// Compiles the rate limit configuration into a Lua table string format.
  fn compile_config(config: &RateLimitConfig) -> String {
    format!(
      "{{ short_window = {{ max = {}, duration_ms = {} }}, long_window = {{ max = {}, duration_ms = {} }}, bucket = {{ capacity = {}, use_cost = {}, duration_ms = {} }} }}",
      config.short_window.max,
      config.short_window.duration.as_nanos() as f64 * 1e-6,
      config.long_window.max,
      config.long_window.duration.as_nanos() as f64 * 1e-6,
      config.bucket.capacity,
      config.bucket.use_cost,
      config.bucket.duration.as_nanos() as f64 * 1e-6
    )
  }

  /// Prepares an invocation of the rate limit script for the given resource and user.
  fn prepare_invoke<'sc>(
    script: &'sc Script,
    resource: &str,
    user: &RateLimitUserId,
    now_ms: f64,
  ) -> ScriptInvocation<'sc> {
    let mut invocation = script.prepare_invoke();
    invocation.key(format!("{{rate-limit:{user}:{resource}}}:updated-at"));
    invocation.key(format!("{{rate-limit:{user}:{resource}}}:bucket"));
    invocation.key(format!("{{rate-limit:{user}:{resource}}}:short-window"));
    invocation.key(format!("{{rate-limit:{user}:{resource}}}:long-window"));
    invocation.arg(now_ms);
    invocation
  }

  /// The script template for the rate limit script.
  const SCRIPT_TEMPLATE: &str = include_str!("use_endpoint.lua");

  pub async fn warm_up(&self, redis: &Redis) -> Result<(), RedisError> {
    let mut conn = redis.conn().clone();

    self.global.load_async(&mut conn).await.ok();
    self.standard.load_async(&mut conn).await.ok();
    for script in self.custom.values() {
      script.load_async(&mut conn).await.ok();
    }

    Ok(())
  }
}

impl From<&'_ SystemRateLimitConfig> for CompiledRateLimiter {
  /// Compiles the rate limit scripts for the given configuration.
  fn from(config: &'_ SystemRateLimitConfig) -> Self {
    let compile_script = |cfg: &RateLimitConfig| {
      let lua_config = format!("local config = {}", Self::compile_config(cfg));
      let script = Self::SCRIPT_TEMPLATE
        .replace("-- CONFIGURATION_PLACEHOLDER", &lua_config);
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
