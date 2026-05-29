use kestrel_postgres::{
  connection::Database, operations::account::get_account_by_email,
};
use rocket::http::StatusClass;
use serde_json::json;

use crate::common::run_with_containers;

mod common;

#[rocket::async_test]
async fn registration_atomicity() {
  run_with_containers(async |containers, client| {
    let username = "john";
    let first_email = "john@example.com";
    let second_email = "john@example.org";
    let password = "loremIpsum123!";

    let body = json!({
      "email": first_email,
      "username": username,
      "password": password,
      "birthday": "2007-05-17",
    });
    let client = client.clone();
    let res = client.post("/auth/register").json(&body).dispatch().await;
    assert_eq!(res.status().class(), StatusClass::Success);

    let body = json!({
      "email": second_email,
      "username": username,
      "password": password,
      "birthday": "2007-05-17",
    });
    let client = client.clone();
    let res = client.post("/auth/register").json(&body).dispatch().await;
    assert_eq!(res.status().class(), StatusClass::ClientError);

    let postgres_url = containers.get_urls().await.postgres;
    let postgres = Database::connect(&postgres_url).await.unwrap();

    let account = get_account_by_email(&postgres, second_email).await;
    assert!(account.is_err());
  })
  .await;
}
