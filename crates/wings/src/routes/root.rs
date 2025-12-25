use rocket::{get, routes, Build, Rocket, State};
use rocket::response::content::RawJson;
use rocket_okapi::{openapi, openapi_get_routes_spec};

#[openapi(tag = "Core")]
#[get("/")]
pub async fn root() -> &'static str {
    "Hello, Kestrel!"
}

/// Serve the generated OpenAPI JSON stored in managed state.
#[get("/openapi.json")]
fn openapi_json(spec: &State<String>) -> RawJson<String> {
    RawJson(spec.inner().clone())
}

/// Mount the routes for this group onto the given Rocket instance.
pub fn mount_routes(_config: &kestrel_config::Config, rocket: Rocket<Build>) -> Rocket<Build> {
    // Generate routes and OpenAPI spec from annotated handlers
    let (generated_routes, openapi) = openapi_get_routes_spec![root];

    // Serialize OpenAPI to a JSON string and store in managed state
    let spec_json = serde_json::to_string(&openapi).expect("Failed to serialize OpenAPI spec");

    rocket
        .manage(spec_json)
        .mount("/", generated_routes)
        .mount("/", routes![openapi_json])
}
