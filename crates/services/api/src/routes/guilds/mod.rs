use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

pub mod create_guild;
pub mod delete_guild;
pub mod get_guild;
pub mod update_guild;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    create_guild::create_guild,
    get_guild::get_guild,
    update_guild::update_guild,
    delete_guild::delete_guild,
  ]
}
