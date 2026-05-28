use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

mod change_password;
mod list_sessions;
mod login;
mod manage_totp;
mod register;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    register::register,
    login::login,
    login::login_mfa,
    change_password::change_password,
    list_sessions::list_sessions,
    manage_totp::enable_totp,
    manage_totp::confirm_enable_totp,
    manage_totp::disable_totp,
  ]
}
