use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

pub mod create_channel;
pub mod get_channel;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    create_channel::create_channel,
    get_channel::get_channel,
  ]
}
