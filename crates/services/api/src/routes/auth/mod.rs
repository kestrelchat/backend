use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

mod change_password;
mod list_sessions;
mod login;
mod register;
pub mod setup_totp;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        register::register,
        login::login,
        login::login_mfa,
        change_password::change_password,
        list_sessions::list_sessions,
        setup_totp::setup_totp,
    ]
}
