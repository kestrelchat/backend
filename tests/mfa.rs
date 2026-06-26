use std::sync::Arc;

use dendryte::adapters::totp::TotpSetup;
use rocket::http::StatusClass;
use serde_json::{Value, json};

use crate::common::{
  TestClient, TokenPair, UserCredentials, bearer_auth, login,
  register_test_users, run_with_containers,
};

mod common;

/// Enrolls a test user into TOTP multi-factor authentication and returns the setup instance.
async fn setup_totp_for(
  client: &Arc<TestClient>,
  user: &UserCredentials,
) -> TotpSetup {
  let session = login(client, user).await;

  let totp_res = client
    .post("/auth/mfa/totp")
    .header(bearer_auth(&session.auth_token))
    .dispatch()
    .await;

  assert_eq!(totp_res.status().class(), StatusClass::Success);
  let totp_body: Value = totp_res.into_json().await.unwrap();
  let totp_secret = totp_body["secret"].as_str().unwrap();

  let totp = TotpSetup::from_secret_base32(totp_secret.to_string()).unwrap();
  let code = totp.generate_current().unwrap();

  let temp_token = totp_body["temp_token"].as_str().unwrap();
  client
    .post("/auth/mfa/totp/confirm")
    .header(bearer_auth(&session.auth_token))
    .json(&json!({ "temp_token": temp_token, "password": user.password, "code": code }))
    .dispatch()
    .await;

  totp
}

/// Initiates the first step of the login challenge for a user with MFA enabled.
async fn initiate_login_mfa(
  client: &Arc<TestClient>,
  user: &UserCredentials,
) -> String {
  let req_body = json!({
    "email": user.email,
    "password": user.password,
    "token": "placeholder"
  });

  let response = client.post("/auth/login").json(&req_body).dispatch().await;

  assert_eq!(
    response.status().class(),
    StatusClass::Success,
    "Initial login failed: {}",
    response.into_string().await.unwrap()
  );

  let res_body: Value = response.into_json().await.unwrap();
  assert_eq!(res_body["status"], "RequiresMfa");
  assert_eq!(res_body["method"], "Totp");

  res_body["temp_token"].as_str().unwrap().to_string()
}

/// Completes the second step of the login challenge using a temporary token and a generated TOTP code.
async fn complete_login_mfa(
  client: &TestClient,
  temp_token: &str,
  code: &str,
) -> TokenPair {
  let mfa_body = json!({
    "temp_token": temp_token,
    "code": code
  });

  let mfa_response = client
    .post("/auth/login/mfa")
    .json(&mfa_body)
    .dispatch()
    .await;

  assert_eq!(
    mfa_response.status().class(),
    StatusClass::Success,
    "MFA verification failed: {}",
    mfa_response.into_string().await.unwrap()
  );

  let mfa_res_body: Value = mfa_response.into_json().await.unwrap();
  assert_eq!(mfa_res_body["status"], "Success");

  TokenPair {
    auth_token: mfa_res_body["auth_token"].as_str().unwrap().to_string(),
    refresh_token: mfa_res_body["refresh_token"].as_str().unwrap().to_string(),
  }
}

#[rocket::async_test]
async fn mfa_login_flow() {
  run_with_containers(async |_, client| {
    // 1. Create a user and enroll them into TOTP
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let totp = setup_totp_for(&client, &user).await;

    // 2. Request initial login to receive temporary MFA token
    let temp_token = initiate_login_mfa(&client, &user).await;

    // 3. Generate current time-based code
    let code = totp.generate_current().unwrap();

    // 4. Complete challenge and acquire access tokens
    complete_login_mfa(&client, &temp_token, &code).await;
  })
  .await;
}

#[rocket::async_test]
async fn disable_totp() {
  run_with_containers(async |_, client| {
    // 1. Register user and enable TOTP
    let user = register_test_users(&client, 1)
      .await
      .unwrap()
      .pop()
      .unwrap();
    let totp = setup_totp_for(&client, &user).await;

    // 2. Initiate login MFA
    let temp_token = initiate_login_mfa(&client, &user).await;
    let code = totp.generate_current().unwrap();
    let session = complete_login_mfa(&client, &temp_token, &code).await;

    // 3. Disable TOTP
    let response = client
      .delete("/auth/mfa/totp")
      .header(bearer_auth(&session.auth_token))
      .json(&json!({"password": user.password}))
      .dispatch()
      .await;
    assert_eq!(response.status().class(), StatusClass::Success);

    // 4. Verify login works without TOTP
    login(&client, &user).await;
  })
  .await;
}
