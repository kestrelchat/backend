use std::sync::Arc;

use rocket::{http::StatusClass, local::asynchronous::Client};
use serde_json::{Value, json};

mod common;

use common::{bearer_auth, login, register_test_users, run_with_containers};

/// Fetches the authenticated user's profile to extract their internal ID.
async fn get_user_id(client: &Arc<Client>, auth_token: &str) -> String {
  let res = client
    .get("/users/@me")
    .header(bearer_auth(auth_token))
    .dispatch()
    .await;

  assert_eq!(
    res.status().class(),
    StatusClass::Success,
    "Failed to fetch user profile for ID extraction"
  );

  let body: Value = res.into_json().await.unwrap();
  body["id"].as_str().unwrap().to_string()
}

#[rocket::async_test]
async fn send_friend_request() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await;
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;

    let user_b_id = get_user_id(&client, &session_b.auth_token).await;

    let res = client
      .post(format!("/users/@me/relationships/{}", user_b_id))
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;

    assert_eq!(
      res.status().class(),
      StatusClass::Success,
      "Failed to send friend request. Response: {}",
      res.into_string().await.unwrap()
    );
  })
  .await;
}

#[rocket::async_test]
async fn cannot_send_request_to_self() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1).await.pop().unwrap();
    let session = login(&client, &user).await;

    let user_id = get_user_id(&client, &session.auth_token).await;

    let res = client
      .post(format!("/users/@me/relationships/{}", user_id))
      .header(bearer_auth(&session.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 400);
  })
  .await;
}

#[rocket::async_test]
async fn duplicate_friend_request_conflict() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await;
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;

    let user_b_id = get_user_id(&client, &session_b.auth_token).await;

    let first_res = client
      .post(format!("/users/@me/relationships/{}", user_b_id))
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;
    assert_eq!(first_res.status().class(), StatusClass::Success);

    let duplicate_res = client
      .post(format!("/users/@me/relationships/{}", user_b_id))
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;

    assert_eq!(duplicate_res.status().code, 409);
  })
  .await;
}

#[rocket::async_test]
async fn accept_friend_request() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await;
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;

    let user_a_id = get_user_id(&client, &session_a.auth_token).await;
    let user_b_id = get_user_id(&client, &session_b.auth_token).await;

    let send_res = client
      .post(format!("/users/@me/relationships/{}", user_b_id))
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;
    assert_eq!(send_res.status().class(), StatusClass::Success);

    let accept_res = client
      .post(format!("/users/@me/relationships/{}", user_a_id))
      .header(bearer_auth(&session_b.auth_token))
      .json(&json!({ "action": "friend" }))
      .dispatch()
      .await;
    assert_eq!(
      accept_res.status().class(),
      StatusClass::Success,
      "Response: {:?}",
      accept_res.into_string().await.unwrap()
    );
  })
  .await;
}
