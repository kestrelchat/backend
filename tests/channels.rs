use rocket::http::StatusClass;
use serde_json::{Value, json};
use std::sync::Arc;

mod common;
use common::{
  TestClient, bearer_auth, login, register_test_users, run_with_containers,
};

/// Helper to get user profile ID
async fn get_user_id(client: &Arc<TestClient>, auth_token: &str) -> String {
  let res = client
    .get("/users/@me")
    .header(bearer_auth(auth_token))
    .dispatch()
    .await;
  let body: Value = res.into_json().await.unwrap();
  body["id"].as_str().unwrap().to_string()
}

#[rocket::async_test]
async fn create_guild_text_channel() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let guild_res = client
      .post("/guilds")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({ "name": "Test Guild" }))
      .dispatch()
      .await;

    let guild_body: Value = guild_res.into_json().await.unwrap();
    let guild_id = guild_body["id"].as_str().unwrap();

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "guild_id": guild_id,
              "identifier": "general",
              "category_id": null
          }
      }))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
  })
  .await;
}

#[rocket::async_test]
async fn create_direct_message() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let user_b_id =
      get_user_id(&client, &login(&client, &users[1]).await.auth_token).await;

    let res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({
          "type": "Direct",
          "data": { "recipient_id": user_b_id }
      }))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_dm_self() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;
    let user_id = get_user_id(&client, &session.auth_token).await;

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Direct",
          "data": { "recipient_id": user_id }
      }))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 400);
  })
  .await;
}

#[rocket::async_test]
async fn create_group_channel() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "My Cool Group" }
      }))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_create_guild_channel_with_invalid_identifier() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let guild_res = client
      .post("/guilds")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({ "name": "Test Guild" }))
      .dispatch()
      .await;
    let guild_body: Value = guild_res.into_json().await.unwrap();
    let guild_id = guild_body["id"].as_str().unwrap();

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "guild_id": guild_id,
              "identifier": "INVALID IDENTIFIER!", // Contains spaces/special chars
              "category_id": null
          }
      }))
      .dispatch()
      .await;

    // Should trigger guild_channels_identifier_format
    assert_eq!(res.status().code, 400);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_create_duplicate_direct_message() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let user_b_id =
      get_user_id(&client, &login(&client, &users[1]).await.auth_token).await;

    // Create the first DM
    client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "type": "Direct", "data": { "recipient_id": user_b_id } }))
      .dispatch()
      .await;

    // Attempt duplicate
    let res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "type": "Direct", "data": { "recipient_id": user_b_id } }))
      .dispatch()
      .await;

    // Should trigger direct_channels_unique_pair
    assert_eq!(res.status().code, 409);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_create_channel_with_nonexistent_guild() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "guild_id": "nonexistent_id",
              "identifier": "general",
              "category_id": null
          }
      }))
      .dispatch()
      .await;

    // Should trigger GUILD_NOT_FOUND (InvalidOperation)
    assert_eq!(res.status().code, 404);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_create_channel_with_empty_display_name() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "" } // Trigger length check
      }))
      .dispatch()
      .await;

    // Should trigger group_channels_display_name_length
    assert_eq!(res.status().code, 400);
  })
  .await;
}
