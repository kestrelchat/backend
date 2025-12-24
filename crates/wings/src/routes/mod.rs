pub mod root;

use rocket::{Build, Rocket};

/// Mount application routes onto the provided Rocket instance.
pub fn mount(rocket: Rocket<Build>) -> Rocket<Build> {
	root::mount_routes(rocket)
}

