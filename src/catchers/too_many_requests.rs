use rocket::{
  Response,
  http::{Header, Status},
  request::Outcome,
  response::Responder,
};

use crate::guards::rate_limit::{HitRateLimit, WithinRateLimit};

struct RateLimitResponder<'inner>(
  &'inner Outcome<WithinRateLimit, Option<HitRateLimit>>,
);

impl<'req> Responder<'req, 'static> for RateLimitResponder<'_> {
  fn respond_to(
    self,
    request: &'req rocket::Request<'_>,
  ) -> rocket::response::Result<'static> {
    match self.0 {
      Outcome::Error((_, Some(hit_rate_limit))) => Ok(
        Response::build()
          .status(Status::TooManyRequests)
          .header(Header::new(
            "Retry-After",
            hit_rate_limit.retry_after().to_string(),
          ))
          .finalize(),
      ),
      _ => Status::TooManyRequests.respond_to(request),
    }
  }
}

/// Responds to a 429 Too Many Requests error.
///
/// Returns a `429 Too Many Requests` response with a `Retry-After` header if the request is rate-limited,
/// or just a `429 Too Many Requests` response otherwise.
#[catch(429)]
pub fn too_many_requests<'req>(
  req: &'req rocket::Request,
) -> impl Responder<'req, 'static> {
  let outcome = req
    .local_cache::<Outcome<WithinRateLimit, Option<HitRateLimit>>, _>(|| {
      Outcome::Error((Status::TooManyRequests, None))
    });
  RateLimitResponder(outcome)
}
