extern crate rocket;
extern crate rocket_okapi;
extern crate serde_json;

pub mod routes;

use rocket::{Build, Rocket, launch};
use std::net::Ipv4Addr;

/// Build and configure the Rocket instance for the web server.
pub async fn web() -> Rocket<Build> {
    let swagger = rocket_okapi::swagger_ui::make_swagger_ui(
        &rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "/openapi.json".to_owned(),
            ..Default::default()
        },
    )
    .into();

    let rocket = rocket::build();

    // Delegate application route mounting to the routes module, then
    // attach the Swagger UI and configure server address/port.
    routes::mount(rocket)
        .mount("/swagger", swagger)
        .configure(rocket::Config {
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            port: 8000,
            ..Default::default()
        })
}

#[launch]
async fn rocket() -> _ {
    web().await
}