use kestrel_server::web;

#[rocket::launch]
async fn rocket() -> _ {
  match web(None).await {
    Ok(rocket) => rocket,
    Err(e) => {
      eprintln!("Failed to initialize Rocket: {}", e);
      std::process::exit(1);
    }
  }
}
