use rocket::{
  Request,
  http::Status,
  request::{FromRequest, Outcome},
};

use crate::database::redis::connection::Redis;
use crate::database::redis::operations::sessions::get_session;
use rocket_okapi::{
  r#gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};

#[derive(Debug, Clone)]
pub struct AuthContext {
  pub user_id: String,
  pub session_id: String,
  pub token: String,
}

impl<'r> OpenApiFromRequest<'r> for AuthContext {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthContext {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    req
      .local_cache_async(async {
        let redis = match req.guard::<&rocket::State<Redis>>().await {
          Outcome::Success(r) => r,
          _ => return Outcome::Error((Status::InternalServerError, ())),
        };

        let token = match req.headers().get_one("Authorization") {
          Some(h) => match h.strip_prefix("Bearer ") {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, ())),
          },
          None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let redis_session = match get_session(redis.inner(), token).await {
          Ok(Some(s)) => s,
          Ok(None) => return Outcome::Error((Status::Unauthorized, ())),
          Err(_) => return Outcome::Error((Status::InternalServerError, ())),
        };

        Outcome::Success(AuthContext {
          user_id: redis_session.account_id,
          session_id: redis_session.session_id,
          token: token.to_string(),
        })
      })
      .await
      .clone()
  }
}
