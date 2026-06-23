use crate::{
  api::guards::auth_context::AuthContext,
  database::redis::{
    connection::Redis,
    operations::rate_limiting::use_endpoint::{
      CompiledRateLimiter, RateLimitUserId,
    },
  },
};
use rocket::{
  http::Status,
  request::{FromRequest, Outcome},
  tokio::join,
};
use rocket_okapi::{
  r#gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};
use std::borrow::Cow;

/// A guard that checks if the request is within the rate limit for the endpoint.
///
/// This guard uses the [`CompiledRateLimiter`] to check if the request is within the rate limit.
///
/// As represented by [`RateLimitUserId`], the user can be identified by:
/// - The authenticated user ID (if available)
/// - The client IP address (if no user ID is available)
#[derive(Debug, Clone, Copy)]
pub struct WithinRateLimit;

/// Represents a rate limit hit, including the number of seconds to wait before retrying.
#[derive(Debug, Clone, Copy)]
pub struct HitRateLimit {
  retry_after: u64,
}

impl HitRateLimit {
  pub fn retry_after(&self) -> u64 {
    self.retry_after
  }
}

impl<'r> OpenApiFromRequest<'r> for WithinRateLimit {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WithinRateLimit {
  type Error = Option<HitRateLimit>;

  async fn from_request(
    req: &'r rocket::Request<'_>,
  ) -> Outcome<Self, Self::Error> {
    *req
      .local_cache_async(async {
        let (rate_limiter, redis, auth_ctx) = join!(
          req.guard::<&rocket::State<CompiledRateLimiter>>(),
          req.guard::<&rocket::State<Redis>>(),
          req.guard::<AuthContext>(),
        );
        let (rate_limiter, redis) = match (rate_limiter, redis) {
          (Outcome::Success(rate_limiter), Outcome::Success(redis)) => {
            (rate_limiter, redis)
          }
          _ => {
            return Outcome::Error((Status::InternalServerError, None));
          }
        };

        let user_id = match &auth_ctx {
          Outcome::Success(auth_ctx) => {
            RateLimitUserId::User(&auth_ctx.user_id)
          }
          _ if let Some(ip) = req.client_ip() => RateLimitUserId::Ip(ip),
          _ => return Outcome::Error((Status::Unauthorized, None)),
        };

        let path = if req.uri().is_normalized() {
          let uri = req.uri();
          Cow::Borrowed(uri.path().as_str())
        } else {
          let uri = req.uri().to_owned().into_normalized();
          Cow::Owned(uri.path().to_string())
        };

        match rate_limiter.use_endpoint(redis, &path, &user_id).await {
          Ok(0) => Outcome::Success(WithinRateLimit),
          Ok(retry_after) => Outcome::Error((
            Status::TooManyRequests,
            Some(HitRateLimit { retry_after }),
          )),
          Err(_) => Outcome::Error((Status::InternalServerError, None)),
        }
      })
      .await
  }
}
