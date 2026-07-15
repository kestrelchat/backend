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

#[rocket::async_test]
async fn get_guild_text_channel() {
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
    let guild_id = guild_body["id"].as_str().unwrap().to_string();

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "guild_id": guild_id,
              "identifier": "general",
              "category_id": null,
          }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
    let body: Value = res.into_json().await.unwrap();
    assert_eq!(body["type"], "Guild");
    assert_eq!(body["data"]["id"], channel_id);
    assert_eq!(body["data"]["guild_id"], guild_id);
    assert_eq!(body["data"]["identifier"], "general");
    assert_eq!(body["data"]["display_name"], "general");
  })
  .await;
}

#[rocket::async_test]
async fn get_direct_channel() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;
    let user_a_id = get_user_id(&client, &session_a.auth_token).await;
    let user_b_id = get_user_id(&client, &session_b.auth_token).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({
          "type": "Direct",
          "data": { "recipient_id": user_b_id }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session_a.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
    let body: Value = res.into_json().await.unwrap();
    assert_eq!(body["type"], "Direct");
    assert_eq!(body["data"]["id"], channel_id);
    assert_eq!(body["data"]["user_a"], user_a_id);
    assert_eq!(body["data"]["user_b"], user_b_id);
  })
  .await;
}

#[rocket::async_test]
async fn get_group_channel() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;
    let user_id = get_user_id(&client, &session.auth_token).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "My Cool Group" }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().class(), StatusClass::Success);
    let body: Value = res.into_json().await.unwrap();
    assert_eq!(body["type"], "Group");
    assert_eq!(body["data"]["id"], channel_id);
    assert_eq!(body["data"]["owner_id"], user_id);
    assert_eq!(body["data"]["display_name"], "My Cool Group");
  })
  .await;
}

#[rocket::async_test]
async fn get_nonexistent_channel_returns_404() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let res = client
      .get("/channels/00000000000000000000000000")
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 404);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_get_guild_channel_as_non_member() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;

    let guild_res = client
      .post("/guilds")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({ "name": "Test Guild" }))
      .dispatch()
      .await;
    let guild_body: Value = guild_res.into_json().await.unwrap();
    let guild_id = guild_body["id"].as_str().unwrap().to_string();

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "guild_id": guild_id,
              "identifier": "general",
              "category_id": null,
          }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session_b.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 404);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_get_direct_channel_as_non_participant() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 3).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;
    let session_c = login(&client, &users[2]).await;
    let user_b_id = get_user_id(&client, &session_b.auth_token).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({
          "type": "Direct",
          "data": { "recipient_id": user_b_id }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session_c.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 404);
  })
  .await;
}

#[rocket::async_test]
async fn cannot_get_group_channel_as_non_owner() {
  run_with_containers(async |_, client| {
    let users = register_test_users(&client, 2).await.unwrap();
    let session_a = login(&client, &users[0]).await;
    let session_b = login(&client, &users[1]).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session_a.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "Private Group" }
      }))
      .dispatch()
      .await;
    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let res = client
      .get(&uri)
      .header(bearer_auth(&session_b.auth_token))
      .dispatch()
      .await;

    assert_eq!(res.status().code, 404);
  })
  .await;
}

#[rocket::async_test]
async fn update_group_channel_display_name() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;
    let user_id = get_user_id(&client, &session.auth_token).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "My Not Cool Group" }
      }))
      .dispatch()
      .await;

    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap().to_string();

    let uri = format!("/channels/{}", channel_id);
    let edit_res = client
      .patch(&uri)
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "Edited Group Name" }
      }))
      .dispatch()
      .await;

    assert_eq!(edit_res.status().class(), StatusClass::Success);

    let edit_body: Value = edit_res.into_json().await.unwrap();
    assert_eq!(edit_body["type"], "Group");
    assert_eq!(edit_body["data"]["id"], channel_id);
    assert_eq!(edit_body["data"]["owner_id"], user_id);
    assert_eq!(edit_body["data"]["display_name"], "Edited Group Name");

    let get_res = client
      .get(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(get_res.status().class(), StatusClass::Success);

    let get_body: Value = get_res.into_json().await.unwrap();
    assert_eq!(get_body["type"], "Group");
    assert_eq!(get_body["data"]["id"], channel_id);
    assert_eq!(get_body["data"]["owner_id"], user_id);
    assert_eq!(get_body["data"]["display_name"], "Edited Group Name");
  })
  .await;
}

#[rocket::async_test]
async fn update_guild_channel() {
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

    let channel_res = client
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

    assert_eq!(channel_res.status().class(), StatusClass::Success);

    let channel_body: Value = channel_res.into_json().await.unwrap();
    let channel_id = channel_body["data"]["id"].as_str().unwrap();

    let uri = format!("/channels/{}", channel_id);
    let edit_res = client
      .patch(&uri)
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Guild",
          "data": {
              "identifier": "staff_only",
              "display_name": "Do not look here!"
          }
      }))
      .dispatch()
      .await;

    assert_eq!(edit_res.status().class(), StatusClass::Success);

    let edit_body: Value = edit_res.into_json().await.unwrap();
    assert_eq!(edit_body["type"], "Guild");
    assert_eq!(edit_body["data"]["id"], channel_id);
    assert_eq!(edit_body["data"]["category_id"], Value::Null);
    assert_eq!(edit_body["data"]["identifier"], "staff_only");
    assert_eq!(edit_body["data"]["display_name"], "Do not look here!");
    assert_eq!(edit_body["data"]["emoji_id"], Value::Null);
    assert_eq!(edit_body["data"]["topic"], Value::Null);

    let get_res = client
      .get(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(get_res.status().class(), StatusClass::Success);

    let get_body: Value = get_res.into_json().await.unwrap();
    assert_eq!(get_body["type"], "Guild");
    assert_eq!(get_body["data"]["id"], channel_id);
    assert_eq!(get_body["data"]["category_id"], Value::Null);
    assert_eq!(get_body["data"]["identifier"], "staff_only");
    assert_eq!(get_body["data"]["display_name"], "Do not look here!");
    assert_eq!(get_body["data"]["emoji_id"], Value::Null);
    assert_eq!(get_body["data"]["topic"], Value::Null);
  })
  .await;
}

#[rocket::async_test]
async fn delete_group_channel() {
  run_with_containers(async |_, client| {
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let session = login(&client, &user).await;

    let create_res = client
      .post("/channels")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({
          "type": "Group",
          "data": { "display_name": "Private Group" }
      }))
      .dispatch()
      .await;

    let create_body: Value = create_res.into_json().await.unwrap();
    let channel_id = create_body["data"]["id"].as_str().unwrap();

    let uri = format!("/channels/{}", channel_id);
    let delete_res = client
      .delete(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(delete_res.status().class(), StatusClass::Success);

    let get_res = client
      .get(&uri)
      .header(bearer_auth(&session.auth_token))
      .dispatch()
      .await;

    assert_eq!(get_res.status().code, 404);
  })
  .await;
}
