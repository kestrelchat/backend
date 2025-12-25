pub mod root;

use rocket::{Build, Rocket};
use kestrel_config::Config;

/// Mount application routes onto the provided Rocket instance.
pub fn mount(config: &Config, rocket: Rocket<Build>) -> Rocket<Build> {
	root::mount_routes(config, rocket)
}

