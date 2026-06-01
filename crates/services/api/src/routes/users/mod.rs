use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

pub mod count_users;
pub mod create_relationship;
pub mod get_self;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    count_users::count_users,
    get_self::get_self,
    create_relationship::create_relationship
  ]
}
