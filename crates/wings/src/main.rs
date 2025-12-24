use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    // test route
    let app = Router::new().route("/", get(|| async { "Hello, Kestrel!" }));

    // run the api server on port 8000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}