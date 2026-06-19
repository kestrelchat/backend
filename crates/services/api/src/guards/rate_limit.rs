use std::borrow::Cow;

use kestrel_redis::{
  connection::Redis,
  operations::rate_limiting::use_endpoint::{
    CompiledRateLimiter, RateLimitUserId,
  },
};
use rocket::{
  Response,
  http::{Header, Status},
  request::{FromRequest, Outcome},
  tokio::join,
};

use crate::utils::auth_context::AuthContext;

/// A guard that checks if the request is within the rate limit for the endpoint.
///
/// This guard uses the [`CompiledRateLimiter`] to check if the request is within the rate limit.
///
/// As represented by [`RateLimitUserId`], the user can be identified by:
/// - The authenticated user ID (if available)
/// - The client IP address (if no user ID is available)
#[derive(Debug, Clone, Copy)]
pub struct WithinRateLimit;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WithinRateLimit {
  type Error = Response<'r>;

  async fn from_request(
    req: &'r rocket::Request<'_>,
  ) -> Outcome<Self, Self::Error> {
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
        return Outcome::Error((
          Status::InternalServerError,
          Response::default(),
        ));
      }
    };

    let user_id = match &auth_ctx {
      Outcome::Success(auth_ctx) => RateLimitUserId::User(&auth_ctx.user_id),
      _ if let Some(ip) = req.client_ip() => RateLimitUserId::Ip(ip),
      _ => return Outcome::Error((Status::Unauthorized, Response::default())),
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
        Response::build()
          .header(Header::new("Retry-After", retry_after.to_string()))
          .finalize(),
      )),
      Err(_) => {
        Outcome::Error((Status::InternalServerError, Response::default()))
      }
    }
  }
}
