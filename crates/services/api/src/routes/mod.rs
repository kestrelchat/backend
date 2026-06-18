pub mod auth;
pub mod guilds;
pub mod meta;
pub mod users;

use rocket::{Build, Rocket};
use rocket_okapi::{
  mount_endpoints_and_merged_docs, openapi_get_routes_spec,
  settings::OpenApiSettings,
};

pub fn mount(mut rocket: Rocket<Build>) -> Rocket<Build> {
  let settings = OpenApiSettings::default();
  mount_endpoints_and_merged_docs!(
      rocket,
      "/".to_owned(),
      settings,
      "/"    => openapi_get_routes_spec![meta::meta],
      "/auth" => auth::routes(),
      "/guilds" => guilds::routes(),
      "/users" => users::routes(),
  );
  rocket
}
