use rocket::Route;
use rocket_okapi::{okapi::openapi3::OpenApi, openapi_get_routes_spec};

mod change_password;
mod list_sessions;
mod login;
mod manage_totp;
mod register;
mod revoke_session;

pub fn routes() -> (Vec<Route>, OpenApi) {
  openapi_get_routes_spec![
    register::register,
    login::login,
    login::login_mfa,
    revoke_session::revoke_current_session,
    change_password::change_password,
    fetch_session::fetch_session,
    list_sessions::list_sessions,
    revoke_session::revoke_all_sessions,
    revoke_session::revoke_session,
    manage_totp::enable_totp,
    manage_totp::confirm_enable_totp,
    manage_totp::disable_totp,
  ]
}
