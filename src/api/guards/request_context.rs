use rocket::{
  Request,
  request::{FromRequest, Outcome},
};
use rocket_okapi::{
  r#gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct RequestContext {
  pub ip: Option<IpAddr>,
  pub user_agent: Option<String>,
}

impl<'r> OpenApiFromRequest<'r> for RequestContext {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestContext {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    Outcome::Success(Self {
      ip: extract_ip(req),
      user_agent: req.headers().get_one("User-Agent").map(str::to_owned),
    })
  }
}

fn extract_ip(req: &Request<'_>) -> Option<IpAddr> {
  if let Some(ip) = req.headers().get_one("CF-Connecting-IP")
    && let Ok(ip) = ip.parse()
  {
    return Some(ip);
  }

  if let Some(forwarded) = req.headers().get_one("X-Forwarded-For")
    && let Some(ip) = forwarded.split(',').next()
    && let Ok(ip) = ip.trim().parse()
  {
    return Some(ip);
  }

  if let Some(ip) = req.headers().get_one("X-Real-IP")
    && let Ok(ip) = ip.parse()
  {
    return Some(ip);
  }

  req.client_ip()
}
