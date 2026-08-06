use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

pub mod add_user_to_channel;
pub mod create_channel;
pub mod delete_channel;
pub mod get_channel;
pub mod update_channel;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    add_user_to_channel::add_user_to_channel,
    create_channel::create_channel,
    get_channel::get_channel,
    update_channel::update_channel,
    delete_channel::delete_channel
  ]
}
